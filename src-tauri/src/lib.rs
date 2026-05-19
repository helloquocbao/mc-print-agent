use axum::{routing::post, Json, Router, http::StatusCode};
use tower_http::cors::{Any, CorsLayer};
use tauri::{AppHandle, Emitter};
use std::net::SocketAddr;
use std::fs::File;
use std::io::Write;
use std::process::Command;

#[derive(serde::Deserialize, Clone, serde::Serialize)]
struct PrintItem {
    #[serde(rename = "type")]
    item_type: String,
    content: String,
}

#[derive(serde::Deserialize, Clone, serde::Serialize)]
struct PrintRequest {
    printer: String,
    data: Vec<PrintItem>,
}

#[derive(serde::Serialize, Clone)]
struct AppLog {
    timestamp: String,
    level: String,
    message: String,
    receipt_preview: Option<String>,
}

// Global reference to emit logs from the Axum threads to the Tauri UI
static mut TAURI_APP: Option<AppHandle> = None;

fn emit_log(level: &str, message: &str, preview: Option<&str>) {
    unsafe {
        if let Some(ref app) = TAURI_APP {
            let log = AppLog {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                level: level.to_string(),
                message: message.to_string(),
                receipt_preview: preview.map(String::from),
            };
            let _ = app.emit("print-log", log);
        }
    }
}

// Helper to strip ESC/POS styling commands for a clean dashboard preview
fn strip_esc_pos(raw: &str) -> String {
    raw.replace("\x1B\x40", "") // Init
        .replace("\x1B\x61\x30", "") // Left
        .replace("\x1B\x61\x31", "") // Center
        .replace("\x1B\x61\x32", "") // Right
        .replace("\x1B\x45\x01", "") // Bold On
        .replace("\x1B\x45\x00", "") // Bold Off
        .replace("\x1B\x64\x04\x1D\x56\x42\x00", "\n--- CẮT GIẤY ---\n") // Feed + Cut
        .replace("\x0A", "\n")
}

// REST HTTP handler for processing print requests from the web app
async fn handle_print(Json(payload): Json<PrintRequest>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut raw_bytes = Vec::new();
    let mut preview_text = String::new();

    for item in &payload.data {
        if item.item_type == "raw" {
            raw_bytes.extend_from_slice(item.content.as_bytes());
            preview_text.push_str(&item.content);
        } else if item.item_type == "image" {
            // Decodes base64 images if present
            use base64::{Engine as _, engine::general_purpose};
            if let Ok(decoded) = general_purpose::STANDARD.decode(&item.content) {
                raw_bytes.extend_from_slice(&decoded);
            }
        }
    }

    let clean_preview = strip_esc_pos(&preview_text);
    emit_log("INFO", &format!("Nhận lệnh in hóa đơn qua máy in: {}", payload.printer), Some(&clean_preview));

    // Handle Native macOS RAW CUPS printing
    #[cfg(target_os = "macos")]
    {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("mc_receipt.bin");
        
        if let Ok(mut file) = File::create(&file_path) {
            if file.write_all(&raw_bytes).is_ok() {
                let mut lp_cmd = Command::new("lp");
                if payload.printer != "default" && !payload.printer.is_empty() {
                    lp_cmd.arg("-d").arg(&payload.printer);
                }
                lp_cmd.arg("-o").arg("raw").arg(&file_path);

                match lp_cmd.output() {
                    Ok(output) => {
                        let _ = std::fs::remove_file(&file_path);
                        if output.status.success() {
                            emit_log("SUCCESS", "Hóa đơn đã được gửi thành công đến System Spooler!", None);
                            return Ok(Json(serde_json::json!({ "success": true, "message": "In thành công" })));
                        } else {
                            let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
                            let friendly_err = if err_msg.contains("No destinations") || err_msg.contains("no default destination") || err_msg.contains("no default printer") || err_msg.contains("no system default") {
                                "Lỗi: Không tìm thấy máy in vật lý nào được cấu hình trên hệ thống. Vui lòng cấu hình máy in trong System Settings -> Printers & Scanners.".to_string()
                            } else {
                                format!("Lỗi lệnh in CUPS: {}", err_msg.trim())
                            };
                            emit_log("ERROR", &friendly_err, None);
                            return Err((StatusCode::INTERNAL_SERVER_ERROR, friendly_err));
                        }
                    }
                    Err(e) => {
                        let _ = std::fs::remove_file(&file_path);
                        let err_msg = format!("Lỗi: Không thể chạy lệnh in 'lp' của hệ thống: {}", e);
                        emit_log("ERROR", &err_msg, None);
                        return Err((StatusCode::INTERNAL_SERVER_ERROR, err_msg));
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&file_path);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Lỗi: Không thể tạo tệp in tạm thời trên đĩa.".to_string()));
    }

    // Windows native raw printing
    #[cfg(target_os = "windows")]
    {
        let target_printer = if payload.printer == "default" || payload.printer.is_empty() {
            match get_windows_default_printer() {
                Some(p) => p,
                None => {
                    let err_msg = "Lỗi: Không tìm thấy máy in mặc định trên Windows. Vui lòng cấu hình máy in mặc định hoặc chọn cụ thể trong Cài đặt.".to_string();
                    emit_log("ERROR", &err_msg, None);
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, err_msg));
                }
            }
        } else {
            payload.printer.clone()
        };

        match print_raw_windows(&target_printer, &raw_bytes) {
            Ok(_) => {
                emit_log("SUCCESS", "Hóa đơn đã được gửi thành công đến Windows Spooler!", None);
                return Ok(Json(serde_json::json!({ "success": true, "message": "In thành công" })));
            }
            Err(e) => {
                let err_msg = format!("Lỗi in trên Windows: {}", e);
                emit_log("ERROR", &err_msg, None);
                return Err((StatusCode::INTERNAL_SERVER_ERROR, err_msg));
            }
        }
    }

    // Linux or other platforms fallback
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        emit_log("SUCCESS", "In hóa đơn thành công (Chế độ giả lập Linux/Khác)", None);
        return Ok(Json(serde_json::json!({ "success": true, "message": "In thành công" })));
    }
}


#[cfg(target_os = "windows")]
fn get_windows_default_printer() -> Option<String> {
    use winapi::um::winspool::GetDefaultPrinterW;
    use winapi::shared::minwindef::DWORD;
    use std::ptr;

    let mut size: DWORD = 0;
    unsafe {
        GetDefaultPrinterW(ptr::null_mut(), &mut size);
    }

    if size == 0 {
        return None;
    }

    let mut buffer: Vec<u16> = vec![0; size as usize];
    unsafe {
        if GetDefaultPrinterW(buffer.as_mut_ptr(), &mut size) != 0 {
            let printer_name = String::from_utf16_lossy(&buffer);
            return Some(printer_name.trim_matches('\0').to_string());
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn print_raw_windows(printer_name: &str, data: &[u8]) -> Result<(), String> {
    use std::ptr;
    use winapi::um::winspool::{OpenPrinterW, StartDocPrinterW, StartPagePrinter, WritePrinter, EndPagePrinter, EndDocPrinter, ClosePrinter, DOC_INFO_1W};
    use winapi::shared::minwindef::DWORD;

    let mut printer_name_wide: Vec<u16> = printer_name.encode_utf16().chain(Some(0)).collect();
    let mut h_printer = ptr::null_mut();

    unsafe {
        if OpenPrinterW(printer_name_wide.as_mut_ptr(), &mut h_printer, ptr::null_mut()) == 0 {
            return Err("Không thể mở kết nối tới máy in. Hãy kiểm tra lại tên kết nối máy in.".to_string());
        }
    }

    struct PrinterGuard(winapi::shared::ntdef::HANDLE);
    impl Drop for PrinterGuard {
        fn drop(&mut self) {
            unsafe {
                ClosePrinter(self.0);
            }
        }
    }
    let _guard = PrinterGuard(h_printer);

    let doc_name_wide: Vec<u16> = "MC Print Agent Job".encode_utf16().chain(Some(0)).collect();
    let datatype_wide: Vec<u16> = "RAW".encode_utf16().chain(Some(0)).collect();

    let mut doc_info = DOC_INFO_1W {
        pDocName: doc_name_wide.as_ptr() as *mut u16,
        pOutputFile: ptr::null_mut(),
        pDatatype: datatype_wide.as_ptr() as *mut u16,
    };

    unsafe {
        let doc_id = StartDocPrinterW(h_printer, 1, &mut doc_info as *mut _ as *mut u8);
        if doc_id == 0 {
            return Err("Không thể khởi động tác vụ in trên Windows spooler.".to_string());
        }

        if StartPagePrinter(h_printer) == 0 {
            let _ = EndDocPrinter(h_printer);
            return Err("Không thể tạo trang in thô (StartPagePrinter).".to_string());
        }

        let mut bytes_written: DWORD = 0;
        let success = WritePrinter(
            h_printer,
            data.as_ptr() as *mut _,
            data.len() as DWORD,
            &mut bytes_written,
        );

        let _ = EndPagePrinter(h_printer);
        let _ = EndDocPrinter(h_printer);

        if success == 0 {
            return Err("Lỗi khi ghi dữ liệu ra máy in.".to_string());
        }
    }

    Ok(())
}


#[tauri::command]
fn get_system_printers() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("lpstat").arg("-e").output() {
            let printers = String::from_utf8_lossy(&output.stdout);
            return printers.lines().map(|s| s.to_string()).collect();
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("powershell")
            .args(&["-NoProfile", "-Command", "Get-Printer | Select-Object -ExpandProperty Name"])
            .output()
        {
            let printers = String::from_utf8_lossy(&output.stdout);
            return printers.lines().map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();
        }
    }
    vec!["default".to_string()]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--silent"])))
        .setup(|app| {
            let handle = app.handle().clone();
            unsafe {
                TAURI_APP = Some(handle);
            }

            // Start HTTP API Server on port 9876 in Tauri's built-in async runtime
            tauri::async_runtime::spawn(async move {
                let cors = CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any);

                let server_app = Router::new()
                    .route("/print", post(handle_print))
                    .layer(cors);

                let addr = SocketAddr::from(([127, 0, 0, 1], 9876));
                
                let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
                axum::serve(listener, server_app).await.unwrap();
            });

            emit_log("INFO", "MC-Print Server đang chạy tại http://localhost:9876", None);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_system_printers])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

