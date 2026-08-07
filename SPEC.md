# Spec: Astral Production-Grade Quality Fixes

## Objective

Bring the Astral Tauri v2 app (Rust + React) to production-grade code quality **without changing user-visible behavior**. Scope is limited to the code-quality defects found in the readiness assessment:

1. CSP `null` in `tauri.conf.json`
2. No tests (Rust or frontend)
3. No CI pipeline
4. Panic-prone `unwrap()` calls and duplicated IPC framing code
5. No logging / observability
6. Version drift (`0.1.0` vs git tag `v2.8.0`)
7. Build artifacts tracked in git (`astral.exe`, `WebView2Loader.dll`); generated `linux-schema.json` untracked and noisy
8. No CHANGELOG
9. Frontend: untyped `invoke` via `window.__TAURI_INTERNALS__`, no ErrorBoundary, misleading default "connected" state, broken favicon reference
10. Hardcoded personal fallbacks in Rust (`telecom.no1`, `C:\Users\Admin\Desktop`)

**Out of scope (explicitly NOT changing):** the hardcoded quest list in `fetch_active_quests` (real quest discovery is a feature, not a quality fix); the spoofer/quest-completion mechanics themselves; all UI styling and layout.

## Tech Stack

- Rust 2021, Tauri v2, tokio, serde/serde_json, dirs
- React 18, TypeScript 5, Vite 5
- Existing deps unchanged except: add `log` + `env_logger` (Rust), add `vitest` (frontend dev-dep only)

## Commands

```
Frontend build:  /usr/bin/npm run build
Frontend test:   /usr/bin/npm test
Rust test:       cargo test          (needs export PATH="$HOME/.cargo/bin:$PATH")
Rust lint:       cargo clippy -- -D warnings
Rust fmt check:  cargo fmt --check
Full build:      /usr/bin/npm run tauri build
```

Note: `npm` on PATH is broken (`Class extends value undefined`); always use `/usr/bin/npm`.

## Project Structure

```
src/lib/tauri.ts          → typed Tauri command wrappers
src/lib/quest.ts          → pure duration/progress helpers (unit-testable)
src/components/           → ErrorBoundary (new)
src-tauri/src/lib.rs      → commands, thin handlers only
src-tauri/src/discord_ipc.rs → shared IPC framing/handshake/SET_ACTIVITY (unit-tested)
src-tauri/src/main.rs     → env_logger init
CHANGELOG.md              → new
.github/workflows/ci.yml  → new
tasks/plan.md, tasks/todo.md → plan artifacts
```

## Code Style

- Rust: `Result<T, String>` for all commands; `?`/`map_err` over `unwrap()`/`expect()`; no panics in production paths.
- IPC framing: single `send_frame(reader, writer, op, payload)` + `read_frame(reader)`; op encoded as u32 LE, then len u32 LE, then payload.
- Frontend: typed `invoke<T>(cmd, args)` helpers; no `any`; no `window.__TAURI_INTERNALS__` checks.
- Keep existing "ponytail:" comment style; add `log::{info,error,warn}` calls for real observability.

## Testing Strategy

- Rust unit tests in `discord_ipc.rs` (frame build/parse round-trip, overflow handling) and `lib.rs` (pure search-filter logic, duration math).
- Frontend unit tests (`vitest`) for `src/lib/quest.ts` pure helpers only — no React DOM tests in this pass.
- Coverage expectation: pure logic paths covered; UI tested via existing manual flows.
- CI runs: `npm ci` → `tsc` → `vite build` → `vitest run`; `cargo fmt --check` → `cargo clippy -D warnings` → `cargo test` on ubuntu-latest and windows-latest.

## Boundaries

- **Always:** verify with build + tests before finishing; keep behavior identical; log real errors; follow existing style.
- **Ask first:** adding new runtime dependencies (only `log`/`env_logger` proposed); changing CI provider; touching the hardcoded quest list.
- **Never:** commit secrets; remove `.gitignore` protections; rewrite spoofer/quest mechanics; change UI visuals.

## Success Criteria

- [ ] `cargo test` passes (new IPC + search tests)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] `tsc && vite build` passes
- [ ] `vitest run` passes
- [ ] CSP set (not `null`) and app still runs
- [ ] Versions synced to `2.8.0` across package.json, Cargo.toml, tauri.conf.json
- [ ] `.gitignore` covers binaries/generated schemas; `astral.exe`/`WebView2Loader.dll` untracked
- [ ] CHANGELOG.md exists with entries
- [ ] CI workflow added
- [ ] No `unwrap()`/`expect()`/panic paths in production code
- [ ] No personal fallbacks (`telecom.no1`, `Admin\Desktop`) in source

## Open Questions

1. Confirm we may add dev/test dependencies (`vitest`, `@types/node`) and Rust `log`/`env_logger`.
2. CI runs on GitHub Actions for repo `nguyenthanhthe/astral` — acceptable to add the workflow file?
3. `git rm --cached astral.exe WebView2Loader.dll` — files stay on disk, just untracked. Acceptable?
