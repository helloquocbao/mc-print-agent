import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import "./App.css";

interface AppLog {
  timestamp: string;
  level: "INFO" | "SUCCESS" | "ERROR";
  message: string;
  receipt_preview?: string;
}

function App() {
  const [printers, setPrinters] = useState<string[]>([]);
  const [selectedPrinter, setSelectedPrinter] = useState<string>("default");
  const [autostart, setAutostart] = useState(false);
  const [logs, setLogs] = useState<AppLog[]>([
    {
      timestamp: new Date().toLocaleTimeString(),
      level: "INFO",
      message: "MC-Print Agent đã được khởi động cục bộ thành công.",
    },
  ]);

  // Load printer list, subscribe to native events, and fetch Autostart status
  useEffect(() => {
    async function loadPrinters() {
      try {
        const list = await invoke<string[]>("get_system_printers");
        setPrinters(list);
      } catch (err) {
        console.error("Không thể lấy danh sách máy in:", err);
      }
    }

    async function checkAutostart() {
      try {
        const active = await isEnabled();
        setAutostart(active);
      } catch (err) {
        console.error("Không thể lấy trạng thái khởi động cùng hệ thống:", err);
      }
    }

    loadPrinters();
    checkAutostart();

    // Listen to real-time logs from Rust axum server thread
    const unlistenPromise = listen<AppLog>("print-log", (event) => {
      setLogs((prev) => [event.payload, ...prev]);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // Toggle Windows / macOS auto-start on boot
  const handleAutostartToggle = async () => {
    try {
      if (autostart) {
        await disable();
        setAutostart(false);
        setLogs((prev) => [
          {
            timestamp: new Date().toLocaleTimeString(),
            level: "INFO",
            message: "Đã tắt tính năng tự động khởi động cùng hệ thống.",
          },
          ...prev,
        ]);
      } else {
        await enable();
        setAutostart(true);
        setLogs((prev) => [
          {
            timestamp: new Date().toLocaleTimeString(),
            level: "SUCCESS",
            message: "Đã kích hoạt tự động khởi động cùng hệ thống thành công!",
          },
          ...prev,
        ]);
      }
    } catch (error: any) {
      setLogs((prev) => [
        {
          timestamp: new Date().toLocaleTimeString(),
          level: "ERROR",
          message: `Cài đặt tự động khởi động thất bại: ${error.message || error}`,
        },
        ...prev,
      ]);
    }
  };

  // Perform a test silent print job to localhost:9876
  const handleTestPrint = async () => {
    try {
      const sampleEscPos = [
        { type: "raw", content: "\x1B\x40" }, // Init
        { type: "raw", content: "\x1B\x61\x31" }, // Align Center
        { type: "raw", content: "\x1B\x45\x01IN THỬ HÓA ĐƠN\x1B\x45\x00\n" }, // Bold Title
        { type: "raw", content: "MERCHANTCONNECT CAFE\n" },
        { type: "raw", content: "\x1B\x61\x30------------------------------------------\n" }, // Align Left + separator
        { type: "raw", content: "Ma don: TEST-12345\n" },
        { type: "raw", content: "Ngay:   " + new Date().toLocaleString("vi-VN") + "\n" },
        { type: "raw", content: "------------------------------------------\n" },
        { type: "raw", content: "1. Capuccino Da (Tai cho)             x1\n" },
        { type: "raw", content: "2. Croissant Bo Toi (Mang ve)          x2\n" },
        { type: "raw", content: "------------------------------------------\n" },
        { type: "raw", content: "\x1B\x61\x31Cam on ban da thu nghiem!\n" },
        { type: "raw", content: "\x1B\x64\x04\x1D\x56\x42\x00" }, // Feed and cut
      ];

      setLogs((prev) => [
        {
          timestamp: new Date().toLocaleTimeString(),
          level: "INFO",
          message: `Đang gửi lệnh in test tới máy in: ${selectedPrinter}...`,
        },
        ...prev,
      ]);

      const response = await fetch("http://localhost:9876/print", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          printer: selectedPrinter,
          data: sampleEscPos,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP Error: ${response.status}`);
      }
    } catch (error: any) {
      setLogs((prev) => [
        {
          timestamp: new Date().toLocaleTimeString(),
          level: "ERROR",
          message: `In thử thất bại: ${error.message || error}`,
        },
        ...prev,
      ]);
    }
  };

  const clearLogs = () => {
    setLogs([]);
  };

  return (
    <div className="dashboard-container">
      {/* Header section with pulsating status LED */}
      <header className="header">
        <div className="brand">
          <div className="brand-logo">🖨️</div>
          <div className="brand-title">
            <h1>MC-PRINT AGENT</h1>
            <p>Hệ thống máy chủ in ấn cục bộ siêu nhẹ</p>
          </div>
        </div>
        <div className="status-badge">
          <div className="status-dot"></div>
          <span className="status-text">Đang chạy (Cổng 9876)</span>
        </div>
      </header>

      {/* Warning banner when no physical printer is found */}
      {printers.length === 0 && (
        <div className="warning-banner">
          <span className="warning-icon">⚠️</span>
          <div className="warning-content">
            <div className="warning-title">Không phát hiện máy in vật lý nào</div>
            <div className="warning-action-text">
              Hệ thống đang chạy ở <strong>chế độ giả lập</strong>. Để in thực tế, vui lòng cắm nguồn/cáp kết nối và thêm máy in trong <strong>Cài đặt hệ thống (Printers & Scanners)</strong> của macOS.
            </div>
          </div>
        </div>
      )}

      {/* Main configuration grid */}
      <section className="config-grid">
        <div className="config-card">
          <div className="card-label">Máy in mặc định hệ thống</div>
          <select
            className="printer-select"
            value={selectedPrinter}
            onChange={(e) => setSelectedPrinter(e.target.value)}
          >
            <option value="default">Máy in mặc định (Default Spooler)</option>
            {printers.map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </div>

        <div className="config-card flex-row justify-between">
          <div className="autostart-info">
            <div className="card-label" style={{ marginBottom: "2px" }}>Khởi động cùng hệ thống</div>
            <span className="card-subtext">Tự động chạy ngầm khi mở máy</span>
          </div>
          <label className="switch">
            <input type="checkbox" checked={autostart} onChange={handleAutostartToggle} />
            <span className="slider round"></span>
          </label>
        </div>

        <div className="config-card col-span-2" style={{ justifyContent: "center" }}>
          <button className="action-btn" onClick={handleTestPrint}>
            <span>⚡</span> In Thử Hóa Đơn
          </button>
        </div>
      </section>

      {/* Real-time terminal log viewer */}
      <section className="terminal-card">
        <div className="terminal-header">
          <span className="terminal-title">Bảng ghi nhận log in ấn trực tiếp (Real-time logs)</span>
          <button className="terminal-clear" onClick={clearLogs}>
            Xóa logs
          </button>
        </div>
        <div className="terminal-body">
          {logs.length === 0 ? (
            <div style={{ color: "rgba(255,255,255,0.2)", textAlign: "center", padding: "20px" }}>
              Không có nhật ký in ấn nào.
            </div>
          ) : (
            logs.map((log, index) => (
              <div key={index} className="log-item">
                <div className="log-meta">
                  <span className="log-time">[{log.timestamp}]</span>
                  <span className={`log-badge ${log.level.toLowerCase()}`}>{log.level}</span>
                </div>
                <div className="log-message">{log.message}</div>
                {log.receipt_preview && (
                  <pre className="receipt-preview">{log.receipt_preview}</pre>
                )}
              </div>
            ))
          )}
        </div>
      </section>
    </div>
  );
}

export default App;

