# Spec: Astral Frontend Production Redesign

## Objective

Redesign the Astral Tauri v2 frontend so it reads as a production-grade desktop
tool, not a vibe-coded demo. The redesign follows the open-design
(nexu-io/open-design) philosophy: a `DESIGN.md` brand contract + design tokens
drive every visual decision, and all user-facing copy is honest, consistent,
and screen-reader friendly.

User role: end customer. Success = a dark, calm, Discord-native-feeling UI where
connection state, quest state, and session state are always visible and
unmistakable; no marketing fluff ("Celestial Edition", "23,800+"), no misleading
labels, no bare `div` click handlers.

**Behavioral contract:** all Rust commands, quest logic in `src/lib/quest.ts`,
and app flows (start quest, stop session, live search, session check) are
preserved exactly. This pass is UI, copy, and architecture only.

## Commands

```
Frontend build:  /usr/bin/npm run build        (tsc && vite build)
Frontend test:   /usr/bin/npm test             (vitest run)
Dev server:      /usr/bin/npm run dev          (vite, port 5173)
Full app build:  /usr/bin/npm run tauri build  (needs PATH="$HOME/.cargo/bin:$PATH")
```

Note: `npm` on PATH is broken; always use `/usr/bin/npm`.

## Tech Stack

- React 18 + TypeScript 5 + Vite 5 (unchanged)
- Plain CSS custom properties (design tokens) — no new runtime deps
- `lucide-react` icons (unchanged)
- `@tauri-apps/api/app#getVersion` for the real version string (removes hardcoded "v2.0")
- No Google Fonts network dependency (system font stack, CSP tightened)

## Project Structure

```
DESIGN.md                     → brand contract (open-design format)
tasks/spec-frontend-redesign.md, tasks/plan.md, tasks/todo.md
src/index.css                 → imports tokens + global styles
src/styles/tokens.css         → design tokens (color/spacing/radius/type/motion)
src/styles/global.css         → reset, base, component styles
src/App.tsx                   → orchestration, state machine, layout
src/components/
  AppHeader.tsx               → brand + live version + connection pill
  SearchInput.tsx             → labeled, keyboard-friendly, clearable
  QuestList.tsx               → list + rows + loading skeleton + empty/error states
  SessionPanel.tsx            → session status, active quest, progress, stop
  ProgressRing.tsx            → accessible progress ring
  Button.tsx                  → primary / secondary / danger / ghost
  StatusPill.tsx              → connected / checking / disconnected
src/lib/quest.ts              → unchanged logic (+ small copy-safe label helpers)
src/lib/quest.test.ts         → existing tests, extended for new helpers
src/lib/tauri.ts              → unchanged typed wrappers
```

## Code Style

- No inline `style=` props for layout/color (semantic classNames + tokens only).
- Every interactive element is a real `<button>`/`<input>`; no clickable `div`s.
- Color conveyed with tokens (`var(--text-muted)`), never raw hex in JSX.
- All user-facing copy in one plain-English voice; no "Celestial"/"Spoofing"/
  marketing numerals. `aria-label` on icon-only controls; `role="status"` +
  `aria-live="polite"` for session messages; `role="progressbar"` + `aria-valuenow`
  on the ring.
- Components under 200 lines; container vs. presentation split preserved.

## Testing Strategy

- `vitest run` (node env) for pure logic only, including new label/formatters in
  `quest.ts` (target duration, remaining, live progress, time format).
- No DOM tests this pass (would need jsdom + testing-library; keep deps minimal).
- Verification: `tsc && vite build` clean; `vitest run` green; manual smoke in
  `npm run dev` (browser) verifying: idle → start → running → stop, live search,
  disconnected/connected pill, empty-search state.

## Boundaries

- **Always:** keep behavior identical; tokens not raw values; a11y + responsive
  verified; tests + build green before finishing.
- **Ask first:** touching Rust commands; changing the quest data model; adding
  runtime dependencies.
- **Never:** commit secrets; remove `.gitignore` protections; rewrite
  spoofer/quest mechanics; keep marketing-fluff copy.

## Success Criteria

- [ ] `DESIGN.md` exists and documents palette/type/spacing/motion/guardrails
- [ ] Tokens live in `styles/tokens.css`; components use tokens only
- [ ] No clickable `div`s; all controls are buttons/inputs with a11y labels
- [ ] Loading, error, empty, connected/disconnected states all rendered
- [ ] "Celestial Edition", "23,800+", "Auto-Execute All", "Spoofing stopped",
      "Orbs & Rewards claimed" no longer appear anywhere in src/ or README
- [ ] App version shown from `getVersion()` (no hardcoded string)
- [ ] Google Fonts removed; CSP tightened accordingly
- [ ] `npm run build` and `npm test` green; behavior unchanged
