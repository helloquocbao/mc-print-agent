# 🖨️ MC-Print Agent

[Tiếng Việt](#-mc-print-agent-tiếng-việt) | [English](#-mc-print-agent-english)

---

## 🇻🇳 MC-Print Agent (Tiếng Việt)

**MC-Print Agent** là một giải pháp mã nguồn mở siêu nhẹ, hiệu năng cao đóng vai trò là "cầu nối" (printing bridge) giữa các ứng dụng Web (như phần mềm bán hàng POS, hệ thống quản lý nhà hàng, quầy thu ngân) và các máy in hóa đơn nhiệt cục bộ sử dụng lệnh ESC/POS.

Được xây dựng trên nền tảng **Tauri + Rust + React + TypeScript**, MC-Print Agent cho phép thực hiện in ấn thô (raw printing) cực nhanh, hoàn toàn im lặng mà không hiển thị hộp thoại in của trình duyệt (silent printing) và tiêu thụ tài nguyên hệ thống vô cùng tiết kiệm.

### ✨ Tính Năng Nổi Bật

- 🚀 **Siêu Nhẹ & Tốc Độ Cực Nhanh**: Khởi động tức thì, dung lượng RAM tiêu thụ cực thấp (thường < 20MB) nhờ hiệu năng tối ưu của ngôn ngữ hệ thống Rust.
- 🔌 **Cổng REST API Cục Bộ (HTTP Server)**: Tích hợp máy chủ Axum chạy ngầm ở cổng `9876`, hỗ trợ đầy đủ CORS, cho phép mọi ứng dụng web bên thứ ba gửi lệnh in trực tiếp qua phương thức `POST /print`.
- 🖨️ **Hỗ Trợ In Thô Chuyên Sâu (Raw ESC/POS Printing)**:
  - **macOS**: Tương tác trực tiếp với hệ thống hàng đợi CUPS cục bộ qua lệnh in thô hệ thống (`lp -d <printer> -o raw <temp_file>`).
  - **Windows**: Giao tiếp trực tiếp với Windows Print Spooler qua Windows API mức thấp (`OpenPrinterW`, `StartDocPrinterW`, `WritePrinter`).
  - **Linux**: Chế độ giả lập in ấn thô ổn định.
- 🖼️ **Giải Mã Hình Ảnh & Dữ Liệu Nhị Phân**: Hỗ trợ nhận diện các lệnh in văn bản thô hoặc hình ảnh/logo đã được mã hóa ở dạng Base64.
- ⚙️ **Quản Lý & Xem Trước Hóa Đơn Trực Quan**: 
  - Giao diện Dashboard tối giản, hiện đại giúp dễ dàng quản lý máy in và theo dõi nhật ký hoạt động.
  - Tự động lọc/loại bỏ các mã điều khiển ESC/POS phức tạp để hiển thị một bản xem trước (preview) dạng văn bản sạch sẽ ngay trên giao diện log của Dashboard.
- 🔄 **Khởi Động Cùng Hệ Thống (Autostart)**: Cấu hình dễ dàng chỉ với một nút gạt để ứng dụng tự động chạy ngầm dưới khay hệ thống khi khởi động máy tính.
- 🔍 **Tự Động Phát Hiện Máy In**: Tự động quét và liệt kê danh sách toàn bộ các máy in vật lý (USB, LAN, Wi-Fi) được cài đặt trên hệ điều hành của bạn.

### 🛠️ Hướng Dẫn Cài Đặt & Phát Triển

#### Yêu Cầu Hệ Thống

Để phát triển hoặc đóng gói dự án này, máy tính của bạn cần cài đặt sẵn:
- **Node.js** (Phiên bản v18 trở lên)
- **Rust & Cargo** (Xem hướng dẫn cài đặt tại [rustup.rs](https://rustup.rs/))
- Các công cụ biên dịch hệ thống (trên Windows cần cài đặt C++ Build Tools qua Visual Studio Installer; trên macOS cần cài đặt Xcode Command Line Tools qua lệnh `xcode-select --install`).

#### Các Bước Thực Hiện

1. **Cloning Repository**
   ```bash
   git clone https://github.com/helloquocbao/mc-print-agent.git
   cd mc-print-agent
   ```

2. **Cài Đặt Thư Viện Frontend**
   ```bash
   npm install
   ```

3. **Chạy Ở Chế Độ Phát Triển (Development Mode)**
   Chạy lệnh sau để khởi chạy giao diện kiểm thử cục bộ và tự động kích hoạt tính năng hot-reload (cập nhật nóng) cho cả mã nguồn Rust và React:
   ```bash
   npm run tauri dev
   ```

4. **Đóng Gói Ứng Dụng (Production Build)**
   Để tạo ra các tệp cài đặt độc lập (`.dmg`/`.app` cho macOS hoặc `.msi`/`.exe` cho Windows):
   ```bash
   npm run tauri build
   ```
   Các tệp cài đặt sau khi đóng gói thành công sẽ nằm trong thư mục `src-tauri/target/release/bundle/`.

### 💻 Hướng Dẫn Tích Hợp API (Dành Cho Nhà Phát Triển Web / POS)

Bất kỳ ứng dụng Web nào cũng có thể gửi lệnh in trực tiếp tới máy in cục bộ thông qua API của **MC-Print Agent** chạy cục bộ ở cổng `9876`.

#### Cấu Trúc API `POST /print`

- **URL**: `http://localhost:9876/print`
- **Method**: `POST`
- **Headers**: `Content-Type: application/json`
- **Body**: 
  - `printer` (String): Tên máy in hệ thống muốn sử dụng. Truyền `"default"` hoặc chuỗi rỗng `""` để sử dụng máy in mặc định của hệ điều hành.
  - `data` (Array): Mảng các phân đoạn dữ liệu in. Mỗi đoạn là một đối tượng chứa:
    - `type` (String): `"raw"` (lệnh ESC/POS dạng chuỗi/nhị phân) hoặc `"image"` (hình ảnh được mã hóa dạng Base64).
    - `content` (String): Nội dung lệnh in thô hoặc chuỗi Base64 hình ảnh.

#### Ví dụ Gọi API Bằng JavaScript (Fetch API)

Dưới đây là đoạn mã mẫu giúp bạn gửi lệnh in một hóa đơn bán lẻ đơn giản tới máy in hóa đơn nhiệt qua máy khách:

```javascript
const handlePrintReceipt = async () => {
  const ESC = "\x1B";
  const GS = "\x1D";
  
  const samplePrintData = [
    { type: "raw", content: ESC + "@" }, // Khởi tạo máy in (Reset)
    { type: "raw", content: ESC + "a\x01" }, // Căn lề giữa (Center Align)
    { type: "raw", content: ESC + "E\x01IN THỬ HÓA ĐƠN\x1B\x45\x00\n" }, // Chữ in đậm
    { type: "raw", content: "CỬA HÀNG MERCHANCONNECT\n" },
    { type: "raw", content: ESC + "a\x00------------------------------------------\n" }, // Căn lề trái + Đường kẻ
    { type: "raw", content: "Mã hóa đơn: HD-88992\n" },
    { type: "raw", content: "Thời gian:   " + new Date().toLocaleString("vi-VN") + "\n" },
    { type: "raw", content: "------------------------------------------\n" },
    { type: "raw", content: "1. Cà Phê Sữa Đá                     x1   29.000đ\n" },
    { type: "raw", content: "2. Bánh Mì Thịt Nướng                x1   35.000đ\n" },
    { type: "raw", content: "------------------------------------------\n" },
    { type: "raw", content: ESC + "E\x01Tổng cộng:                          64.000đ\x1B\x45\x00\n" },
    { type: "raw", content: "------------------------------------------\n" },
    { type: "raw", content: ESC + "a\x01Cảm ơn quý khách! Hẹn gặp lại.\n" },
    { type: "raw", content: "\x0A\x0A\x0A" }, // Đẩy giấy lên trước khi cắt
    { type: "raw", content: ESC + "d\x04" + GS + "V\x42\x00" }, // Cuộn giấy và Cắt giấy (Feed & Paper Cut)
  ];

  try {
    const response = await fetch("http://localhost:9876/print", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        printer: "default", // Hoặc điền tên chính xác của máy in trong cấu hình hệ thống của bạn
        data: samplePrintData,
      }),
    });

    const result = await response.json();
    if (response.ok && result.success) {
      console.log("Hóa đơn đã được gửi đến spooler hệ thống thành công!");
    } else {
      console.error("Lỗi in ấn từ MC-Print Agent:", result.message || "Không rõ nguyên nhân");
    }
  } catch (error) {
    console.error("Không thể kết nối đến máy khách MC-Print Agent cục bộ:", error);
  }
};
```

### ⚖️ Tuyên Bố Miễn Trừ Trách Nhiệm (Disclaimer)

> [!WARNING]
> **VUI LÒNG ĐỌC KỸ TRƯỚC KHI SỬ DỤNG PHẦN MỀM**

1. **Cung cấp "Như hiện có" (AS IS)**: Đây là một dự án phần mềm mã nguồn mở được chia sẻ miễn phí vì mục đích cộng đồng. Phần mềm này được cung cấp trên cơ sở nguyên trạng "như hiện có" mà không có bất kỳ hình thức bảo đảm hay cam kết rõ ràng hoặc ngầm định nào, bao gồm nhưng không giới hạn ở các bảo đảm về khả năng hoạt động thương mại, tính chính xác hoặc mức độ tương thích trên thiết bị của bạn.
2. **Miễn trừ hoàn toàn trách nhiệm pháp lý**: Tác giả, người đóng góp và nhà phát triển dự án này hoàn toàn **không chịu bất kỳ trách nhiệm nào** trước pháp luật hay thực tế đối với bất kỳ khiếu nại, tổn thất, thiệt hại trực tiếp, gián tiếp, ngẫu nhiên hoặc là hậu quả phát sinh từ việc cài đặt, biên dịch, vận hành hoặc sử dụng phần mềm này dưới mọi hình thức.
3. **Mọi rủi ro thuộc về người dùng**: 
   - Nếu xảy ra bất kỳ sự cố nào liên quan đến **phần cứng** (như hỏng hóc máy in hóa đơn, cháy nổ cổng kết nối, kẹt giấy, hao mòn đầu in nhiệt/đầu phun kim nhanh chóng), **phần mềm** (xung đột driver hệ thống, lỗi hàng đợi in spooler, treo hệ điều hành), hoặc **tổn thất kinh doanh** (mất mát dữ liệu doanh số, gián đoạn hoạt động bán hàng/thanh toán tại quầy cashier), **bạn là người duy nhất tự chịu hoàn toàn trách nhiệm và mọi rủi ro**.
   - Nhà phát triển sẽ không cung cấp bất kỳ hỗ trợ bồi thường tài chính, sửa chữa thiết bị hay đền bù chi phí khắc phục hậu quả nào cho người dùng.
4. **Lời khuyên vận hành**: Chúng tôi đặc biệt khuyên nghị người dùng cần thực hiện chạy thử nghiệm phần mềm kỹ lưỡng trong môi trường thử nghiệm (sandbox) với máy in thực tế của mình để xác nhận tính ổn định trước khi đưa vào áp dụng chính thức cho các hoạt động sản xuất kinh doanh trực tiếp.

### 📄 Giấy Phép (License)

Dự án này được phân phối công khai dưới dạng mã nguồn mở theo Giấy phép **MIT**. Bạn hoàn toàn được phép tự do chỉnh sửa, sao chép, phân phối và sử dụng thương mại hóa tùy theo mục đích cá nhân và doanh nghiệp của mình theo các điều khoản chung của giấy phép này.

---

## 🇺🇸 MC-Print Agent (English)

**MC-Print Agent** is a lightweight, high-performance open-source printing bridge designed to connect web-based applications (such as cloud POS software, restaurant management systems, and cashier terminals) with local thermal receipt printers using ESC/POS commands.

Built on **Tauri + Rust + React + TypeScript**, MC-Print Agent enables lightning-fast, quiet raw printing without browser dialog prompts (silent printing), while consuming minimal system resources.

### ✨ Features

- 🚀 **Ultra-lightweight & Fast**: Instant startup and low resource consumption (typically under 20MB of RAM) powered by the efficiency of Rust.
- 🔌 **Local REST API (HTTP Server)**: Features an integrated Axum server running on port `9876` with full CORS support. Any third-party web application can send print commands directly via `POST /print`.
- 🖨️ **Native Raw ESC/POS Printing**:
  - **macOS**: Communicates directly with the local CUPS queue via native system print commands (`lp -d <printer> -o raw <temp_file>`).
  - **Windows**: Interfaces directly with the Windows Print Spooler via low-level Windows APIs (`OpenPrinterW`, `StartDocPrinterW`, `WritePrinter`).
  - **Linux**: Fallback mode for reliable raw printing simulation.
- 🖼️ **Image & Binary Decoding**: Supports printing raw text commands as well as Base64-encoded binary images (e.g., logo, barcodes).
- ⚙️ **Visual Dashboard & Live Logs**: 
  - Minimalist, modern dashboard UI to manage selected default printers and monitor status.
  - Automatically filters/strips complex ESC/POS styling characters to display a clean text preview of receipts directly in the log pane.
- 🔄 **Run on Boot (Autostart)**: Easily configure the app to run minimized in the background on system boot with a single click.
- 🔍 **Automatic Printer Discovery**: Automatically scans and lists all physical printers (USB, network LAN, Wi-Fi) configured on your operating system.

### 🛠️ Installation & Development Guide

#### System Requirements

To build or package this project, you need:
- **Node.js** (v18 or higher)
- **Rust & Cargo** (Follow instructions on [rustup.rs](https://rustup.rs/))
- System compilation tools (C++ Build Tools for Windows; Xcode Command Line Tools for macOS: `xcode-select --install`).

#### Step-by-Step Setup

1. **Clone the Repository**
   ```bash
   git clone https://github.com/helloquocbao/mc-print-agent.git
   cd mc-print-agent
   ```

2. **Install Frontend Dependencies**
   ```bash
   npm install
   ```

3. **Run in Development Mode**
   Start the local testing UI with hot-reloading for both Rust and React:
   ```bash
   npm run tauri dev
   ```

4. **Build and Package Application**
   Compile and package stand-alone installers (`.dmg`/`.app` for macOS or `.msi`/`.exe` for Windows):
   ```bash
   npm run tauri build
   ```
   The compiled packages will be located in `src-tauri/target/release/bundle/`.

### 💻 API Integration Guide (For Web / POS Developers)

Any web application can trigger printing by calling the local **MC-Print Agent** HTTP server running on port `9876`.

#### API Specification `POST /print`

- **Endpoint**: `http://localhost:9876/print`
- **Method**: `POST`
- **Headers**: `Content-Type: application/json`
- **Body Schema**:
  - `printer` (String): The exact name of the destination printer. Send `"default"` or an empty string `""` to use the operating system's default printer.
  - `data` (Array): Array of printing items. Each item must contain:
    - `type` (String): `"raw"` (text ESC/POS commands) or `"image"` (Base64-encoded image).
    - `content` (String): The raw print commands or Base64 string content.

#### Integration Example in JavaScript (Fetch API)

Use the following JavaScript boilerplate to send a ticket directly to the printer:

```javascript
const handlePrintReceipt = async () => {
  const ESC = "\x1B";
  const GS = "\x1D";
  
  const samplePrintData = [
    { type: "raw", content: ESC + "@" }, // Initialize printer
    { type: "raw", content: ESC + "a\x01" }, // Align Center
    { type: "raw", content: ESC + "E\x01TEST RECEIPT PRINT\x1B\x45\x00\n" }, // Bold Text
    { type: "raw", content: "CAFE\n" },
    { type: "raw", content: ESC + "a\x00------------------------------------------\n" }, // Left align + divider
    { type: "raw", content: "Order ID:   HD-88992\n" },
    { type: "raw", content: "Timestamp:  " + new Date().toLocaleString() + "\n" },
    { type: "raw", content: "------------------------------------------\n" },
    { type: "raw", content: "1. Iced Milk Coffee                 x1   $1.99\n" },
    { type: "raw", content: "2. Grilled Pork Banh Mi             x1   $2.99\n" },
    { type: "raw", content: "------------------------------------------\n" },
    { type: "raw", content: ESC + "E\x01Total Amount:                       $4.98\x1B\x45\x00\n" },
    { type: "raw", content: "------------------------------------------\n" },
    { type: "raw", content: ESC + "a\x01Thank you! See you again.\n" },
    { type: "raw", content: "\x0A\x0A\x0A" }, // Feed paper
    { type: "raw", content: ESC + "d\x04" + GS + "V\x42\x00" }, // Feed and Cut paper
  ];

  try {
    const response = await fetch("http://localhost:9876/print", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        printer: "default", // Or your target printer name on system settings
        data: samplePrintData,
      }),
    });

    const result = await response.json();
    if (response.ok && result.success) {
      console.log("Print job successfully sent to system spooler!");
    } else {
      console.error("Print error from agent:", result.message || "Unknown error");
    }
  } catch (error) {
    console.error("Failed to connect to local MC-Print Agent:", error);
  }
};
```

### ⚖️ Disclaimer

> [!WARNING]
> **PLEASE READ THIS SECTION CAREFULLY BEFORE USING OR DEPLOYING THE SOFTWARE**

1. **Provided "AS IS"**: This is a free, open-source software project shared for public use. The software is provided on an "AS IS" basis, without warranty of any kind, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose, or non-infringement.
2. **Absolute Limitation of Liability**: In no event shall the authors, contributors, or copyright holders be liable for any claim, damages, or other liability, whether in an action of contract, tort, or otherwise, arising from, out of, or in connection with the software or the use or other dealings in the software.
3. **User Assumes All Risks**: 
   - You assume **100% of all risks** and responsibilities regarding **hardware damages** (including but not limited to receipt printer breakdown, ports blowout, paper jams, thermal print head burnout), **software issues** (driver conflicts, print queue locking, operating system crashes), and **business losses** (data loss, transaction failures, cashier station downtime).
   - The developer provides absolutely no hardware repairs, replacement compensation, or troubleshooting support for business disruptions.
4. **Best Practices**: We highly recommend testing this software thoroughly in a sandbox/staging environment with your specific printer models to verify compatibility and stability before running it in a production live environment.

### 📄 License

This project is licensed under the **MIT License**. You are free to modify, distribute, and commercially use this project, provided the original copyright notice and license are included.
