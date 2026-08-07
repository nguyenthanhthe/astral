# Spec: Update check + GitHub link + README rewrite

## Objective
- Backend: new `check_for_update` command that queries the GitHub releases API
  for the latest astral tag and reports whether a newer version exists.
- Frontend: a "Check for updates" affordance in the header (with result state:
  up-to-date / new version available → link to release) and a GitHub logo that
  opens the astral repo via the shell plugin.
- Rewrite `README.md` to match the current architecture (Rust services layer,
  session engine, Tauri v2, current release assets).
- **No new release** — everything stays local (master).

## Acceptance criteria
1. `check_for_update` returns `UpdateInfo { latest_version, current_version,
   is_update_available, url }`; version comparison is a pure, unit-tested
   function that ignores a leading `v` and compares dot-segments numerically.
2. Offline/HTTP failure returns a typed `AppError` (code `UPDATE_CHECK_FAILED`)
   — never a panic.
3. Header shows: GitHub logo (opens `https://github.com/nguyenthanhthe/astral`)
   and an update button/pill that on click checks and renders
   idle → checking → available (link to the release) / up-to-date / error.
4. README reflects current stack (Rust services, session engine/events,
   `catalog://updated`, build-from-source with `npm run tauri build`), current
   release assets (v2.10.0: deb/rpm/exe/msi/dmg), and the new update check.
5. `cargo test`, `cargo clippy --all-targets -- -D warnings`, `npm test`,
   `npm run build` all stay green.

## Commands
- Rust: `cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings`
- FE: `npm test` and `npm run build`
- Live: `cd src-tauri && RUST_LOG=info timeout 30 cargo run` (Discord running)

## Files
- `src-tauri/src/infra/config.rs` — repo/release URLs
- `src-tauri/src/services/update/mod.rs` — pure version compare + reqwest fetch
- `src-tauri/src/app/error.rs` — `UPDATE_CHECK_FAILED` variant
- `src-tauri/src/lib.rs` — `check_for_update` command + register
- `src/lib/tauri.ts` — `checkForUpdate()` + `UpdateInfo`
- `src/components/AppHeader.tsx` + `src/App.tsx` — update state + GitHub link
- `src/styles/` — header update pill styles (token-based)
- `README.md` — rewrite
- `CHANGELOG.md` — entry (kept under `[Unreleased]`, no release)

## Boundaries
- Always: TDD for the version-compare pure function; token-based CSS.
- Never: open external URLs outside the shell plugin; hardcode versions in FE.
- Ask first: none (small, self-contained, local-only).
