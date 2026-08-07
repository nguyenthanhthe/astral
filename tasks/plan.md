# Plan: Astral Frontend Production Redesign

## Order

1. **Design tokens** (`src/styles/tokens.css`) — palette, spacing, radius,
   typography, motion. Everything downstream consumes these.
2. **Global styles** (`src/styles/global.css`) — reset, base, focus-visible,
   reduced-motion, component classes. `src/index.css` re-exports both.
3. **Pure helpers** (`src/lib/quest.ts` + tests) — add `labelForQuestTarget`,
   `labelForProgress`, `labelForSession` helpers; extend tests.
4. **Primitives** — `Button`, `StatusPill`.
5. **Composite** — `AppHeader`, `SearchInput`, `QuestList`, `ProgressRing`,
   `SessionPanel`.
6. **App shell** (`src/App.tsx`) — state machine + layout + error banner,
   wired to the same `tauri.ts` commands.
7. **Chrome** — `index.html` (drop Google Fonts), `tauri.conf.json` (tighten
   CSP), `main.tsx` (import stylesheet once).
8. **Docs** — `DESIGN.md`, `README.md` copy, `CHANGELOG.md` entry.
9. **Verify** — `npm run build`, `npm test`, grep for banned copy.

## Dependencies

- 1 → 2 → 3 → 4 → 5 → 6 (sequential; components import tokens/styles)
- 7 and 8 are independent of 5/6; 9 gates completion.

## Risks

- **Behavior drift** → keep every `invoke` call and state variable; only relabel.
- **CSP breakage** → removing Google Fonts requires dropping the two CSP
  `https://fonts.*` directives together, else style-src blocks nothing new but
  fonts gstatic is orphaned; verify build + dev.
- **npm broken on PATH** → always `/usr/bin/npm`.

## Checkpoints

- After 3: `npm test` green with new helpers.
- After 6: `npm run build` green (tsc strict).
- After 9: full grep sweep + both commands green.
