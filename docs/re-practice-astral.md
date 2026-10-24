# RE Practice — phân tích chính binary Astral (walkthrough)

> Mục tiêu luyện tập RE trên **sản phẩm của chính mình** — hợp lệ 100%, không đụng đến
> binary của bên thứ ba. Target: binary harness của `astral_lib` (debug build, GNU toolchain):
>
> - `src-tauri/target/debug/deps/astral_lib-9aee2c5572c6fca8.exe`
> - SHA-256: `5ed3ef0ca2e92d1207b91dc9c5dd2dffe427be32cb00b3a1d265e00f148247be`
> - PE x86_64, chứa toàn bộ logic Astral (session engine, queststore, catalog, IPC).

## 0. Môi trường phân tích trên máy này

| Thành phần | Vị trí | Ghi chú |
| :--- | :--- | :--- |
| Ghidra 12.1.4 | `D:\rea-tools\ghidra\ghidra_12.1.4_PUBLIC` | khớp `GHIDRA_INSTALL_DIR` |
| Temurin JDK 21.0.12.1+1 | `D:\rea-tools\jdk\jdk-21.0.12.1+1` | khớp `JAVA_HOME` |
| rea MCP | `npx rea-agents@6.4.0 mcp` | cấu hình trong `~/.zcode/cli/config.json` |

Lưu ý: server rea **memoize provider health lúc khởi động**. Sau khi (re)cài Ghidra/JDK,
restart phiên ZCode để rea nhận provider; kiểm tra bằng `npx rea-agents@6.4.0 doctor`.
Nếu server rea cache cũ, chạy Ghidra headless trực tiếp (mục 3.3) — kết quả như nhau.

## 1. Lộ trình 4 bước khi phân tích một PE (áp dụng cho Astral)

### Bước 1 — Nhận diện tổng thể (`rea` → `binary_overview`)

Mở binary trong rea (`open_binary` với path tuyệt đối), rồi `binary_overview` trả về:
format (PE), kiến trúc (x86_64), số segment và **tổng số procedure + string**. Với một
binary Rust debug, con số procedure sẽ rất lớn (std + tokio + toàn bộ dependency) —
bình thường; bước 2 mới thu hẹp phạm vi.

### Bước 2 — Định vị code của mình bằng string anchoring

Không đọc 30k hàm. Dùng **chuỗi log/label đặc trưng làm mỏ neo**:

- `search_strings` pattern `Completing Quest` → địa chỉ literal trong `.rdata`.
- `search_strings` pattern `Earning ` → literal của `format!("Earning {}", ...)`.
- `search_strings` pattern `custom_` → prefix id của Quest Mirror (`queststore::new_id`).

Xref ngược từ địa chỉ string (`xrefs` / `find_xrefs_to_name`) → trúng đúng hàm chứa nó
trong `services::session::engine`. Đây là kỹ thuật nhanh nhất để "thả mồi" vào code của
mình giữa một đống std của Rust.

### Bước 3 — Đọc decompilation (`procedure_pseudo_code`)

`search_procedures` pattern `session` lọc tên hàm (Rust debug giữ symbol
`astral_lib::services::session::engine::*`). Với hàm trúng mồi, gọi
`procedure_pseudo_code` — Ghidra decompile ra C-like pseudocode. Đối chiếu với source
trong repo để *đọc ngược*: match arms của `launch()`, vòng `tokio::select!` của `run()`.

### Bước 4 — Chứng minh luồng dữ liệu (`procedure_callees` / `procedure_references`)

Từ `run()` → callees thấy `hold_ipc_activity` → `activity::hold_activity` → các call
`WriteFile` lên socket IPC. Đây là cách chứng minh binary KHÔNG gọi gì ngoài local IPC —
kiểm chứng chính sách "no token, no network" của Astral ngay trên binary.

## 2. Kết quả thực chạy (Ghidra headless 12.1.4 + script `AstralDump.java`)

Tham số chạy: import + DWARF ≈ 9 phút, auto-analysis cap `-analysisTimeoutPerFile 600`,
script dump < 1 phút. Hai phát hiện chính:

### 2.1. String anchoring THẤT BẠI trên binary Rust — và vì sao

`search_strings` cho `Completing Quest` / `Earning ` / `custom_` trả về **0 kết quả**,
dù chuỗi chắc chắn nằm trong binary. Nguyên nhân: Rust lưu literal như **byte span
(ptr, len) không null-terminated**, Ghidra auto-analysis không đánh dấu chúng là string
data. Đây chính là caveat ghi trong tài liệu rea ("Ghidra may omit … including Rust
strings; an empty result does not prove text is absent"). Bài học: trên binary Rust,
**đừng neo vào string — neo vào symbol/DWARF**.

### 2.2. Symbol anchoring trúng đích ngay (6 hàm, bằng chứng DWARF)

Ghidra khôi phục tên đầy đủ từ DWARF, giúp chứng minh đúng code của Astral nằm ở đâu:

| Hàm trong binary | Địa chỉ | Nghĩa là gì |
| :--- | :--- | :--- |
| `serde_core::private::size_hint::cautious<astral_lib::services::queststore::CustomQuest>` | `0x140003610` | Type `CustomQuest` của Quest Mirror được compile vào binary — đây là đường serde của `custom_quests.json` |
| `tokio::task::spawn::spawn_inner<astral_lib::services::session::engine::run::{async_fn_env#0}>` | `0x140006420` | `engine::run()` được spawn thành tokio task — khớp `tauri::async_runtime::spawn(run(...))` trong `engine.rs` |
| `spawn_inner` / `spawn` / `{closure#0}` cùng generic `engine::run` | `0x140006270` → `0x1400068c0` | Cả chuỗi plumbing runtime của task session, nguyên vẹn qua decompile |

Snippet thật từ decompile (rút gọn):

```c
JoinHandle<()> __rustcall
tokio::task::spawn::spawn_inner<astral_lib::services::session::engine::run::_async_fn_env_0_>
          (undefined8 future, undefined8 param_2)
{
  IStack_c18 = tokio::runtime::task::id::Id::next();
  ...
  tokio::runtime::context::current::with_current<...join::JoinHandle<()>>(...);
  if ((auStack_430[0] & 1) != 0) {
    /* panic: TryCurrentError — spawn ngoài runtime */
    ::core::panicking::panic_display<tokio::runtime::handle::TryCurrentError>(&TStack_1c);
  }
  return (JoinHandle<()>)(Header *)auStack_430._8_8_;
}
```

Đọc ngược được đúng ngữ nghĩa source: cấp `Id` cho task, đăng ký vào runtime hiện tại,
panic nếu không có runtime — và future được nhắm tới chính là `session::engine::run`.

### 2.3. Việc còn lại khi bạn mở Ghidra GUI

Từ những địa chỉ trên: `Function Call Graph` của `0x140006420` sẽ kéo tới
`hold_ipc_activity` → `activity::hold_activity` → các call `WriteFile` lên
`\\.\pipe\discord-ipc-0` — cách kiểm chứng binary chỉ nói chuyện với Discord qua local
IPC, không có token nào bị đọc.

## 3. Replay bằng Ghidra headless (khi không dùng rea MCP)

```bash
mkdir -p /d/Temp/ghidra_proj   # analyzeHeadless yêu cầu thư mục tồn tại sẵn
export JAVA_HOME="D:/rea-tools/jdk/jdk-21.0.12.1+1"
"/d/rea-tools/ghidra/ghidra_12.1.4_PUBLIC/support/analyzeHeadless.bat" \
  D:/Temp/ghidra_proj astral \
  -import "D:/Project/astral/src-tauri/target/debug/deps/astral_lib-9aee2c5572c6fca8.exe" \
  -analysisTimeoutPerFile 600 \
  -scriptPath "D:/Temp/ghidra_scripts" -postScript AstralDump.java
```

Script `D:/Temp/ghidra_scripts/AstralDump.java` làm đúng 2 việc tương đương bước 2–3 ở
trên: tìm string anchor + decompile các hàm `session::engine` / `queststore`.

> Lưu ý cú pháp: dùng **forward slashes** cho mọi đường dẫn truyền vào `.bat` —
> backslash bị cmd ăn mất (`D:\Temp` → `Temp`). Với binary debug 200MB, đặt
> `-analysisTimeoutPerFile` để analyzer *Decompiler Parameter ID* không kéo analysis
> quá nửa giờ.

## 4. Bài học rút ra

- **String anchoring** là mỏ neo rẻ nhất khi binary quá lớn — chọn chuỗi chỉ code của
  mình có ("Completing Quest: ", "custom_") để lọc khỏi std Rust.
- **Binary debug GNU** mang full DWARF + symbol Rust: Ghidra mất thời gian import DWARF
  (chục phút cho 200MB) nhưng bù lại tên hàm giữ nguyên — tốt cho luyện đọc.
- Binary chạy được ≠ analysis xong: analysis headless lưu project sau khi xong;
  đợi `Analysis succeeded` trước khi script chạy.
