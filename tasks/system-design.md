# System Design: Tái kiến trúc Backend Astral (Rust + Tauri)

> Version: draft-1 · Ngày: 2026-08-07
> Kết quả của skills `api-and-interface-design` (contract-first) và
> `planning-and-task-breakdown` (vertical slicing).

---

## 1. Bối cảnh & Vì sao phải tái thiết kế

Astral kế thừa từ **Orbshacker** (Python). Kỹ thuật lõi đúng: mở socket IPC của
Discord, SET_ACTIVITY có `[start, end]` timestamps, và (Windows) spawn các
`*.exe` alias để lừa process scanner. Nhưng phần "xương" được **dịch nguyên
cách nghĩ của Python** sang Rust — đúng nghĩa đen, chỉ đổi ngôn ngữ chứ không
đổi kiến trúc. Hậu quả:

| # | Triệu chứng (code thật đang tồn tại) | Vì sao là "Python-copy" |
|---|--------------------------------------|--------------------------|
| 1 | `Cargo.toml` khai `tokio = { features = ["full"] }` nhưng **không một `async` nào** trong code | Python hay đồng bộ + thread; Rust/Tauri sinh ra để async |
| 2 | Fetch Discord detectable database bằng **`powershell.exe Invoke-RestMethod`** (`preload_detectable_cache`) | Đây là chạy Python-script bên trong Rust. Rust có `reqwest` |
| 3 | 625 dòng monolithic trong `lib.rs`: command + cache + IPC + spoofer + search trộn một file | Python script hay viết 1 file lớn; Rust cần module hoá |
| 4 | Global mutable state: `static DETECTABLE_CACHE: Mutex<Option<Vec<serde_json::Value>>>` | Python `global`; Tauri có `State` quản lý vòng đời |
| 5 | Mọi command trả `Result<_, String>` | Python `raise Exception(str)`; cần typed error contract |
| 6 | Frontend **tự đếm ngược bằng timer**; backend im lặng | Python không có event channel; Tauri có `emit()` |
| 7 | JSON dựng bằng `format!(r#"..."#)` — không escape, `serde_json` chỉ dùng để parse | Python f-string; Rust có `serde_json::json!` |
| 8 | `stop_spoofer` chạy `taskkill /f /im <name>` — giết **mọi** tiến trình trùng tên của user | Python `os.system`; Rust phải quản lý PID |
| 9 | Staging spoofer vào `Desktop/Win64`; path PowerShell cứng `C:\Windows\...` | Python hay rải file lộn xộn; nên dùng app-data dir |
| 10 | `optimize_ram()` gọi `SetProcessWorkingSetSize(MAX)` — hack "vibe" có thể hại perf | Không phải thiết kế, là mẹo |

**Kết luận:** giữ nguyên lõi kỹ thuật (không đổi cơ chế), nhưng tái kiến trúc để
Rust/Tauri thực sự được phát huy: async + typed + event-driven + module hoá.

---

## 2. Mục tiêu & Nguyên tắc

1. **Async-native.** Mọi I/O (socket IPC, HTTP catalog, process) chạy trên tokio.
2. **Backend = nguồn chân lý (single source of truth).** Progress/session do
   backend tính và **đẩy event**, frontend chỉ render. Bỏ timer frontend.
3. **Typed mọi nơi.** Thay chuỗi thần chú (`"[Console Quest]"`, `"700 Orbs"`,
   `"astral_1"`, client_id mặc định) bằng enum/struct.
4. **Contract-first.** Command + event + error là hợp đồng rõ ràng, không đổi
   breaking khi chưa có migration.
5. **An toàn khi chạy.** Không `taskkill /im`, không ghi vào Desktop, không panic
   (bỏ `panic = "abort"` nếu giữ thread, hoặc dùng async).
6. **Không thay đổi hành vi người dùng** trong giai đoạn đầu (vẫn mở được quest,
   vẫn search, vẫn progress) — chỉ đổi "đường ống" bên dưới.

---

## 3. Kiến trúc tổng thể

```
┌──────────────────────────────────────────────────────────────┐
│  FRONTEND (React/TS)                                          │
│  - subscribe events → render state                            │
│  - invoke commands (typed wrapper, giữ src/lib/tauri.ts)      │
└──────────────────────────┬───────────────────────────────────┘
                           │ Tauri IPC: invoke + listen
┌──────────────────────────▼───────────────────────────────────┐
│  COMMAND LAYER (thin)  — validate input, call service,        │
│                         trả AppResult<T>                      │
├───────────────────────────────────────────────────────────────┤
│  app/error.rs     AppError: typed codes + message             │
│  app/state.rs     AppState (tauri::State, không global Mutex) │
├───────────────────────────────────────────────────────────────┤
│  domain/ (thuần, không I/O, unit-test được)                   │
│    quest.rs  session.rs  catalog.rs  reward.rs  target.rs     │
├───────────────────────────────────────────────────────────────┤
│  services/ (chứa I/O, async)                                  │
│    discord/connection.rs  — connect/retry/reconnect + events  │
│    discord/ipc.rs         — framing + typed payloads          │
│    catalog/game_catalog.rs— reqwest fetch, TTL, validate      │
│    session/engine.rs      — sở hữu session đang chạy + events │
│    spoofer/orchestrator.rs— PID-tracked (Windows)             │
│    memory/trimmer.rs      — optional RAM trim (Windows)       │
├───────────────────────────────────────────────────────────────┤
│  platform/  cfg-gated: windows.rs unix.rs                     │
├───────────────────────────────────────────────────────────────┤
│  infra/  config.rs  logging.rs                                │
└───────────────────────────────────────────────────────────────┘
```

Quy tắc phụ thuộc: `domain ← services ← command layer ← tauri`. Domain không
biết gì về tauri/io; service không biết gì về frontend.

---

## 4. Tận dụng Rust & Tauri đúng cách (Hiện tại → Tương lai)

| Hạng mục | Hiện tại (Python-copy) | Tương lai (Rust-native) |
|---|---|---|
| Async runtime | Không dùng dù khai tokio | `async fn` command + tokio task cho engine/catalog/connection |
| HTTP | `powershell Invoke-RestMethod` | `reqwest` (thêm dep runtime nhỏ, bỏ hẳn PowerShell) |
| State | `static Mutex<Option<Vec<Value>>>` | `tauri::State<AppState>` (managed, borrow-checker giám sát) |
| UI cập nhật | Frontend poll + timer riêng | `app_handle.emit("session://progress", …)` event-driven |
| JSON | `format!` nối chuỗi | `serde_json::json!` + typed structs |
| Lỗi | `Result<_, String>` | `AppError` enum + code máy đọc được |
| Process | `Command::spawn` rồi quên PID | Track `HashSet<Pid>`; kill đúng PID; cleanup file app-data |
| Config | Hằng số rải rác (nonce, client_id, reward) | `infra/config.rs` tập trung, có thể user chỉnh sau |
| Tauri feature | `tray-icon` bật nhưng không dùng | dùng (minimize-to-tray) hoặc bỏ feature |

---

## 5. Domain model (typed, thay "stringly-typed")

```rust
// domain/target.rs — thay chuỗi thần chú "[Console Quest]"
pub enum LaunchTarget {
    Exe { exe_name: String },
    Console,                       // PS5 / Xbox
    Stream,                        // Voice Channel Stream
}
impl LaunchTarget {
    pub fn label(&self) -> String { /* "Endfield.exe" | "Console (PS5 / Xbox)" | "Voice stream" */ }
}

// domain/quest.rs
pub struct Quest {
    pub id: QuestId,               // branded: QuestId(String)
    pub title: String,
    pub game_name: String,
    pub target: LaunchTarget,
    pub client_id: String,         // brand qua client_id để khỏi nhầm với id khác
    pub reward: Reward,
    pub saved_percent: u8,         // 0..=100, tự clamp
}
pub enum Reward { Orbs(u32), Other(String) }  // serialize về String cho FE

// domain/session.rs
pub enum SessionKind { Exe(LaunchTarget), Console, Stream }
pub struct Session {
    pub id: SessionId,
    pub quest: Quest,
    pub kind: SessionKind,
    pub started_at: Instant,
    pub target_sec: Duration,
    pub initial_percent: u8,
}
impl Session {
    pub fn progress(&self, now: Instant) -> u8 { /* 1 nguồn duy nhất tính % */ }
}

// domain/catalog.rs — thay Vec<serde_json::Value> (untyped, untrusted)
pub struct DetectableGame {
    pub name: String,
    pub client_id: String,
    pub executables: Vec<String>,
}
```

> Quy tắc boundary (từ api-and-interface-design): dữ liệu từ
> `discord.com/api/v9/applications/detectable` là **untrusted** → validate + skip
> record hỏng ngay khi parse, không để `serde_json::Value` chảy vào domain.

---

## 6. IPC Contract (contract-first)

### 6.1 Commands (frontend gọi `invoke`)

Tất cả đều `async fn`, trả `AppResult<T>`.

| Command | Input | Output | Ghi chú |
|---|---|---|---|
| `check_discord_session` | — | `DiscordStatus` | đọc state + nudge reconnect |
| `fetch_active_quests` | — | `Vec<DiscordQuest>` | quests hardcoded mặc định |
| `search_discord_games` | `{ query }` | `Vec<DiscordQuest>` | catalog typed, cap 25 |
| `refresh_catalog` | — | `CatalogState` | refetch bằng reqwest |
| `check_for_update` | — | `UpdateInfo` | GitHub latest release vs running version |
| `start_session` | `{ quest: DiscordQuest }` | `()` | engine; lỗi `SESSION_ACTIVE` nếu đang chạy |
| `stop_session` | — | `()` | idempotent |
| `get_session_status` | — | `SessionStarted \| null` | re-hydrate sau reload |
| `set_settings` | `SettingsPatch` | `Settings` | additive, optional fields |
| `get_settings` | — | `Settings` | |
| `optimize_ram` | — | `String` | trim working set (Windows) |

### 6.2 Events (backend → frontend, `listen`)

| Event | Payload | Khi nào |
|---|---|---|
| `discord://status` | `DiscordStatus { connected, username, user_id }` | connect/disconnect/reconnect |
| `session://started` | `SessionStarted { session_id, quest_id, game_name, exe_name, target_sec, initial_percent }` | bắt đầu quest |
| `session://progress` | `{ session_id, percent, elapsed_sec, remaining_sec }` | mỗi giây từ engine |
| `session://finished` | `{ session_id, quest_id }` | hết giờ |
| `session://stopped` | `{ session_id, reason: "USER" \| "ERROR" }` | user stop / lỗi |
| `catalog://updated` | `{ count, source: "network" \| "cache", at }` | catalog refresh |

### 6.3 Error semantics (1 mẫu duy nhất, không trộn)

```rust
// app/error.rs
pub enum AppError {
    DiscordNotReachable,      // code DISCORD_NOT_REACHABLE
    SessionActive,           // code SESSION_ACTIVE
    QuestNotFound,           // code QUEST_NOT_FOUND
    PlatformUnsupported,     // code PLATFORM_UNSUPPORTED  (spoofer ngoài Windows)
    CatalogEmpty,            // code CATALOG_EMPTY
    Validation(String),      // code VALIDATION
    Internal(String),        // code INTERNAL  (log chi tiết, KHÔNG đẩy ra FE)
}
```
Serialize thành:
```json
{ "code": "DISCORD_NOT_REACHABLE", "message": "Discord isn't running." }
```
FE chỉ render `message`; `code` để logic (vd disable nút Start).

---

## 7. Thiết kế từng service

### 7.1 `services/discord/ipc.rs` (giữ nguyên framing, typed payload)
- Giữ `encode_frame`/`decode_frame`/`ReadWrite`/`unix_socket_path` (đã đúng).
- **Đổi** `handshake`/`set_activity`: dựng payload bằng `serde_json::json!`
  (escape an toàn, hết lỗi JSON do tên game có `"`).
- Trả typed: `HandshakeResult { username, user_id }` thay vì `serde_json::Value`.
- Thêm test: handshake payload đúng shape, tên game chứa `"` vẫn hợp lệ.

### 7.2 `services/discord/connection.rs` (async, self-healing)
- Một tokio task quản lý 1 kết nối: connect → handshake → (chờ) → phát hiện mất
  kết nối → retry với backoff (`200ms, 500ms, 1s, 2s … cap 10s`).
- Trạng thái lưu trong `AppState`, mỗi lần đổi → `emit("discord://status")`.
- Loại bỏ việc `check_discord_session` tự `preload_detectable_cache()` (tách mối lo).

### 7.3 `services/catalog/game_catalog.rs` (bỏ PowerShell)
- `reqwest` GET `https://discord.com/api/v9/applications/detectable` trong tokio
  task khi app start + `refresh_catalog`.
- Parse → `Vec<DetectableGame>`, **validate từng record** (name/client_id bắt buộc),
  bỏ record hỏng, đếm số hợp lệ.
- TTL refresh (mặc định 24h) + cache trả về ngay khi chưa xong fetch.
- Search: `quest_matches` đổi thành nhận `&str` + index name (đủ cho 25k record,
  vẫn O(n) contains nhưng typed và không lọc cả serde_json::Value từng field).

### 7.4 `services/session/engine.rs` (nguồn chân lý về progress)
- `start_session`: kiểm tra `session::Option` trong AppState (rỗng thì chạy),
  spawn tokio task:
  1. áp dụng launch path (Exe → spoofer; Console/Stream → IPC activity với
     `[start,end]`);
  2. vòng lặp `interval 1s`: `session.progress(now)` → `emit session://progress`;
  3. hết `target_sec` → clear activity → `emit session://finished` → xoá state.
- `stop_session`: `abort()` task tương ứng, gọi spoofer cleanup, emit stopped.
- Frontend **xoá timer + logic `currentProgress`** (giữ helper chỉ để fallback),
  chỉ subscribe event.

### 7.5 `services/spoofer/orchestrator.rs` (Windows, an toàn)
- Spawn: lưu `Pid` vào `AppState.spoofer_pids: HashSet<u32>`.
- Stop: `taskkill /f /pid <pid>` **chỉ đúng PID đã spawn** — không `/im` (tránh
  giết game/user đang chơi thật).
- Staging: thư mục app-data (`dirs::data_dir()/astral/spoof`) thay cho
  `Desktop/Win64`; cleanup xoá đúng file đã copy.
- `#[cfg(windows)]` toàn bộ module; các nền tảng khác trả
  `PlatformUnsupported` (hành vi giữ nguyên).

### 7.6 `services/memory/trimmer.rs`
- `optimize_ram` → xoá hoặc chuyển thành setting `memory.trim_on_start` (mặc
  định **tắt**) gọi 1 lần sau khi cửa sổ render xong; không call mỗi giây.
- Giữ `#[cfg(windows)]`, các nền khác no-op.

### 7.7 `infra/config.rs`
- Tập trung: default client_id cho custom quest, nonce prefix, TTL catalog,
  target durations (30s/15m), reward mặc định, spoof staging dir, backoff.
- Settings dùng `tauri-plugin-store` (add) hoặc file JSON đơn giản tự quản
  (không add dep) — khuyến nghị **file JSON tự quản** ở giai đoạn 1.

---

## 8. State management

```rust
// app/state.rs
pub struct AppState {
    pub discord: RwLock<SessionState>,       // thay Mutex static
    pub catalog: RwLock<Option<Catalog>>,
    pub session: RwLock<Option<Session>>,
    pub spoofer: RwLock<SpooferRegistry>,    // HashSet<Pid>
    pub settings: RwLock<Settings>,
    pub app_handle: AppHandle,               // để emit events
}
```
Đăng ký một lần trong `tauri::Builder::setup()`:
```rust
let state = AppState::new(app.handle());
app.manage(state);
tokio::spawn(connection_task(app.handle()));
tokio::spawn(catalog_task(app.handle()));
```
Không còn `static` global — lifecycle do Tauri quản lý, test dễ (tạo AppState
thuần).

---

## 9. Độ tin cậy & Bảo mật

- **Không panic** ở production path: bỏ `panic = "abort"` (đang kèm
  `std::thread::spawn` → panic 1 thread chết cả app) hoặc thay hết bằng async +
  `AppResult`.
- **Boundary validation**: catalog từ mạng phải qua validate; input command qua
  `FromStr`/clamp trước khi chạm domain.
- **Không rò internal error**: `Internal(String)` chỉ vào log, FE nhận message
  an toàn.
- **Process an toàn**: kill theo PID; không giết theo tên.
- **CSP/network**: fetch catalog chỉ 1 host cố định; không mở shell plugin cho
  URL user nhập (giữ `tauri-plugin-shell` scope hẹp hoặc bỏ).
- **Reconnect**: mất Discord → event "Not connected" → tự nối lại, không cần
  restart.

---

## 10. Lộ trình migration (vertical slices)

Mỗi slice để lại app **chạy được**. `npm run tauri build` giữ làm cổng kiểm tra.

### Phase 0 — Nền tảng
- [x] **T1** `app/error.rs` + `AppResult<T>` + tests. *(Verify: cargo test)*
- [x] **T2** `app/state.rs` + `infra/config.rs` + wiring `setup()`;
      xoá `static DETECTABLE_CACHE`. *(Verify: cargo clippy -D warnings)*
- [x] **T3** `domain/` models (Quest/LaunchTarget/Session/Reward) + migration
      khỏi string markers + tests. *(Verify: cargo test)*

### Phase 1 — Discord connectivity
- [x] **T4** `discord/ipc.rs` typed payload (`serde_json::json!`) + tests
      (tên game chứa `"`). *(Verify: cargo test)*
- [x] **T5** `discord/connection.rs` async + retry + `emit discord://status`.
      *(Verify: chạy app, mở/tắt Discord thấy pill đổi)*
      - Connection task là owner duy nhất; I/O blocking qua `spawn_blocking`;
        `check_discord_session` đọc state + wake task bằng `Notify` khi đang
        disconnected. FE thêm subscription `discord://status` (chỉ pill, chưa
        phải migration timer T9). Verified live: "Discord IPC connected as
        meomaybekkk_02260" trên Linux với Discord đang chạy.

### Phase 2 — Catalog (xoá PowerShell)
- [x] **T6** `catalog/game_catalog.rs`: reqwest + validate + TTL;
      bỏ `preload_detectable_cache` (powershell). *(Verify: search hoạt động,
      log "catalog updated N games" — không còn powershell trong ps)*
      - `reqwest` 0.13.4 `default-features=false, features=["json","default-tls"]`
        (openssl có sẵn, nasm thiếu nên không dùng aws-lc-rs). `parse_games` tách
        khỏi HTTP để test; fixture `detectable_sample.json` (LoL/Endfield/Genshin
        thật). `Catalog` typed: `search`/`find`/`is_fresh`; task `spawn` fetch khi
        khởi động + mỗi TTL. Verified live: log `catalog updated: 23907 games`.
- [x] **T7** Search qua catalog typed + tests. *(Verify: cargo test, tìm
      "Genshin" ra kết quả)*
      - `search_discord_games` dùng `Catalog::search` (case-insensitive, cap
        SEARCH_LIMIT); thêm `refresh_catalog` command (Result<CatalogState,
        AppError>); `merge_catalog_hits` tách ra test được (dedupe + fallback
        custom quest). Tests: 57 pass, clippy `-D warnings` sạch.

### Phase 3 — Session engine (event-driven)
- [x] **T8** `session/engine.rs`: start/stop, interval 1s, progress events,
      finish/stop events. *(Verify: app chạy, progress tự chạy không cần timer FE)*
      - 1 tokio task sở hữu session (session được lưu `AppState.session` +
      `session_task` handle stop signal). Events: `session://started`,
      `session://progress` (1s), `session://finished`, `session://stopped`
      (reason `USER`/`ERROR`). Launch path: Exe → spoofer (catalog exes) + IPC
      activity; Console/Stream → IPC activity với `[start,end]`. Ticks bằng
      `tokio::select!` + `watch::channel` để stop sạch.
- [x] **T9** Async command layer + FE chuyển từ timer sang `listen` event,
      giữ `tauri.ts` wrapper. *(Verify: npm test + build + manual smoke)*
      - Commands mới: `start_session`, `stop_session`, `get_session_status`
        (re-hydrate sau reload). FE xoá `setInterval` + `currentProgress`/`remainingSec`;
        progress là state đẩy từ engine. `quest.ts` giữ `formatTime`/
        `questTargetLabel`; các helper timer đã bỏ. Tests: 68 Rust + 5 Vitest.

### Phase 4 — Spoofer hardening (Windows)
- [x] **T10** `spoofer/orchestrator.rs`: PID tracking, kill theo PID, staging
      trong app-data; bỏ `taskkill /im` + `Desktop/Win64`. *(Verify: manual trên
      Windows)*
      - `exe_names_for_simulation(catalog, game)` → `win32_exe_names()` từ
        catalog (bỏ alias hardcode eve/WWM); `sanitize_exe_name` chống path
        traversal (`/` và `\`); staging `app_data_dir()/spoof`; `stop_all`
        kill từng PID trong `SpooferRegistry` rồi `remove_dir_all` staging.
- [x] **T11** `memory/trimmer.rs` gating. *(Verify: clippy/test)*
      - FFI `SetProcessWorkingSetSize` dời từ lib.rs vào
        `services/memory/trimmer.rs` (no-op ngoài Windows).

### Phase 5 — Contract & polish
- [x] **T12** `set_settings`/`get_settings`, contract doc, CHANGELOG, CI giữ
      xanh. *(Verify: npm test, cargo test, build)*
      - `SettingsPatch` additive; wrapper FE `getSettings`/`setSettings`.
- [x] **T13** Update check + GitHub link + README rewrite. *(Verify: cargo test,
      clippy, npm test, live smoke)*
      - `services/update/mod.rs`: `check_for_update` → GitHub latest-release
        API; `version_is_newer` thuần (dot-segment, bỏ `v` prefix, segment
        non-số → 0); lỗi `UPDATE_CHECK_FAILED` không leak detail.
      - Header: pill "Check for updates" (idle/checking/up-to-date/available →
        link release) + logo GitHub mở repo qua shell plugin.
      - README viết lại theo kiến trúc hiện tại; CHANGELOG Unreleased; giữ
        local, không phát hành.

**Checkpoint mỗi 2 task:** `cargo test` + `cargo clippy -D warnings` + `npm run
build` xanh, chạy được bằng binary release.

---

## 11. Rủi ro & Giảm thiểu

| Rủi ro | Ảnh hưởng | Giảm thiểu |
|---|---|---|
| Đổi contract làm hỏng FE | Cao | Contract-first (mục 6) + wrapper `tauri.ts` giữ tên; FE migration 1 lần ở T9 |
| reqwest + tokio làm tăng binary | Thấp (thêm ~200KB) | Không thêm plugin HTTP; chỉ 1 dep runtime; giữ LTO/opt-level z |
| Bỏ PowerShell thay bằng reqwest khi mạng chặn | Thấp | Giữ fallback: cache cũ + custom quest (hành vi như cũ) |
| Spoofer không test được trên Linux | Cao (chỉ Windows) | cfg-gate + unit test cho logic thuần; manual checklist riêng Windows |
| Task kill sai | Cao | Chỉ kill PID đã spawn, không `/im`; registry lưu path từng PID |

---

## 12. Quyết định mặc định (chốt trước khi code)

1. Bỏ `optimize_ram` mặc định (chuyển thành setting tắt).
2. Không add `tauri-plugin-store` giai đoạn 1 — config file JSON tự quản.
3. Giữ `reqwest` là dep runtime duy nhất mới (bỏ PowerShell dependency gián tiếp).
4. Event naming dùng namespace `discord://`, `session://`, `catalog://`.
5. `panic = "abort"` bỏ hoặc chỉ giữ khi 0 thread thủ công.
6. Hành vi user-visible giữ nguyên cho tới Phase 3 (FE sẽ thấy UI cập nhật
   mượt hơn nhờ event, không mất chức năng).
