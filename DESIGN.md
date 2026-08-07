# Astral Design System

> Category: Productivity & Desktop Tool
> Discord Quest & Activity automation. Dark-first, Discord-native feel, calm and
> precise. The UI exists to answer one question at a glance: "what is running,
> and how far along is it?"

## 1. Visual Theme & Atmosphere

Astral is a companion tool for Discord's Quest program. The surface is
dark-first and quiet: three-step depth (`#111214` → `#1e1f22` → `#2b2d31`) so
panels separate by tone, not shadow. The signature **Blurple** (`#5865f2`) —
Discord's own brand color — is reserved for the one thing that matters: the
primary action. Everything else is neutral so the connection pill and the
running quest stay instantly scannable.

Typography is a system UI stack (no network font dependency): geometric,
legible at 13–15px, with a monospace stack reserved for numbers (countdowns,
percentages, process names). Headings step incrementally; meta text is 12px
muted. Density is compact like a chat client — nothing decorative, every pixel
carries information.

Shapes: 8px cards, 4–6px controls, full pills for status. 1px dividers at low
alpha. No glow, no heavy shadows, no gradients-as-decoration. The brand mark
uses one flat blurple star.

**Key Characteristics:**
- Dark surfaces `#111214` / `#1e1f22` / `#2b2d31` (3-step depth)
- Single saturated accent: Blurple `#5865f2`, used only for primary actions
- System font stack; mono for numbers and process identifiers
- Compact density; status visible at a glance (pill top-right)
- Semantic status colors: online green, idle yellow, danger red, offline gray
- Pixel-snapped 1px dividers at low alpha; flat surfaces, no glow

## 2. Color Palette & Roles

### Brand
- **Blurple** `#5865f2` — primary actions, focus, selected state
- **Blurple Hover** `#4752c4` — hover/active for primary actions
- **Blurple Soft** `rgba(88, 101, 242, 0.12)` — selected-row wash, tags

### Surfaces (dark, default)
- **Background Base** `#111214` — window background
- **Background Raised** `#1e1f22` — panels, cards
- **Background Overlay** `#2b2d31` — nested surfaces, inputs, hover rows
- **Background Sunken** `#0d0e10` — well-inset areas, progress track
- **Border Subtle** `rgba(255, 255, 255, 0.06)` — dividers
- **Border Default** `#3f4147` — solid card/input borders

### Text
- **Header Primary** `#f2f3f5` — page/brand headings
- **Text Normal** `#dbdee1` — body
- **Text Muted** `#949ba4` — meta, timestamps, hints
- **Text Disabled** `#5c6066` — inactive states
- **Text Link** `#00a8fc` — hyperlinks

### Status & Semantic
- **Success** `#23a55a` — connected, completed
- **Warning** `#f0b232` — idle, degraded
- **Danger** `#f23f43` — disconnected-critical, destructive
- **Info** `#5865f2` — neutral emphasis, running session

## 3. Typography Rules

### Font Families
- **UI / Body / Headings**: system stack
  `-apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue", Arial, sans-serif`
- **Numbers / Process names**: monospace stack
  `"SF Mono", ui-monospace, "Cascadia Code", Consolas, monospace`

### Hierarchy

| Role | Size | Weight | Line Height | Notes |
|------|------|--------|-------------|-------|
| Brand / Page title | 15px | 600 | 1.25 | Header left, with version |
| Section title | 13px | 600 | 1.25 | Uppercase, muted, letterspaced |
| Body / Quest name | 15px | 500 | 1.3 | Primary row text |
| Meta / caption | 12px | 400 | 1.3 | Muted |
| Mono number | 13px | 500 | 1.2 | Countdown, percentage, target |

### Principles
- Weight + color (muted vs normal) carry hierarchy — no invented sizes.
- Body never below 13px; density comes from line-height, not size.
- Numbers always mono so they stop jumping width during countdown.

## 4. Component Stylings

### Buttons
- **Primary**: bg `#5865f2`, text white, 4px radius, padding 8px 16px, hover `#4752c4`
- **Secondary**: bg `#4e5058`, text `#dbdee1`, hover `#6d6f78`
- **Ghost**: transparent, text `#dbdee1`, hover bg `rgba(255,255,255,0.06)`
- **Danger**: bg `#f23f43`, text white, hover `#c72e34`
- Disabled: bg `rgba(255,255,255,0.08)`, text `#5c6066`, no pointer events
- Focus: 2px outline `#5865f2`, offset 2px

### Inputs
- Background `#1e1f22`, border 1px `#3f4147`, radius 4px, padding 8px 12px
- Placeholder `#5c6066`; focus border `#5865f2` + 2px ring at 30% alpha
- Clear button inside right edge, ghost icon 16px

### Quest Rows
- Background `#1e1f22`; hover `rgba(255,255,255,0.03)`; radius 8px; border 1px `rgba(255,255,255,0.06)`
- Selected/running: border `#5865f2`, bg `rgba(88,101,242,0.08)`
- Left: name (15px/500) + meta (12px muted). Right: reward + progress label
- Progress bar: 4px track `rgba(255,255,255,0.08)`, fill blurple, radius full

### Status Pill
- Connected: bg `rgba(35,165,90,0.12)`, border `rgba(35,165,90,0.35)`, dot `#23a55a`
- Checking: bg `rgba(240,178,50,0.10)`, border `rgba(240,178,50,0.3)`, dot `#f0b232`
- Disconnected: bg `rgba(128,132,142,0.12)`, border `rgba(128,132,142,0.35)`, dot `#80848e`
- Dot 8px, no glow; text 12px 500

### Cards / Panels
- Background `#1e1f22`, border 1px `rgba(255,255,255,0.06)`, radius 8px
- Padding 16–20px; internal gap 12–16px

## 5. Spacing & Layout

- **Base unit**: 4px. Scale: 4, 8, 12, 16, 20, 24, 32, 40.
- Two-column workspace (≥ 880px): quest list (flex 3) + session panel (fixed 320px)
- Below 880px: single column, session panel after list
- Window padding 16px; header 56px with bottom 1px divider
- Progress ring 128px on desktop, 112px on small windows

## 6. Motion

- Hover: 120ms `ease-out`
- Ring stroke: 300ms `ease-out` (never animated on `prefers-reduced-motion`)
- Status transitions: 200ms fade
- No looping, pulsing, or marquee animations

## 7. Usage Guardrails

- Blurple is for the *primary* action and active state only; never decorate
  headings, badges, or idle UI with it.
- Color never carries information alone — every status has text or an icon
  (e.g. the pill says "Connected · username", not just a green dot).
- No marketing numerals ("23,800+"), no "Celestial"-style flavor names, no
  claims the UI does not actually perform.
- Every interactive element must be a real button/input; clickable divs are
  banned. Keyboard focus must be visible.
- Numbers and durations render in the mono stack; nothing may jump width while
  a countdown ticks.
- Preserve the dark shell and compact density; a light theme or balloon-radius
  treatment breaks the Discord-native feel.
