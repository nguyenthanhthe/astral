# Astral Design System — "Starlight Observatory"

> Category: Productivity & Desktop Tool
> Redesign (v2.13.0). The previous Discord-gray world is retired: Astral is
> not a Discord clone, it is an **observatory console** — a dark instrument
> panel pointed at Discord's quest sky. One accent family (starlight cyan)
> means *action & progress*; one reserved gold means *orbs (rewards)*.
> Everything else is deep-space ink so the running session is the brightest
> thing on screen.

## 1. Visual Theme & Atmosphere

Deep-space ink surfaces with a blue cast (never pure black, never
gray-on-gray): `#0a0c12` → `#10131c` → `#171c28`. Depth is carried by
**starlit edges** — each panel has a hairline border plus a 1px inner top
highlight, as if lit from above — not by drop shadows.

The signature moment is the **Orbit**: the session progress ring renders as
an orbital arc with a cyan→violet gradient stroke and a comet-head dot at its
tip, glowing softly while the session runs. A quiet constellation SVG anchors
the empty session state. Reward moments use the reserved gold, never cyan.

Typography stays on the system stack (no network fonts). Numerals —
countdowns, percentages, process names — are mono and never shift width while
ticking. Section labels are 11px uppercase micro-caps with 0.10em tracking.

**Key characteristics:**
- Deep-space ink surfaces `#0a0c12` / `#10131c` / `#171c28` with starlit top edges
- Single interactive accent: starlight cyan `#4cc2ff`; violet `#8b7cf6` only inside gradients
- Reward gold `#f0c649` reserved for orbs/rewards — never for actions
- Orbit ring with gradient stroke + comet head; constellation empty state
- Compact density; mono numerals; text always carries status, never color alone

## 2. Color Palette & Roles

### Brand
- **Starlight** `#4cc2ff` — primary actions, focus, progress, selected state
- **Starlight Hover** `#6fd0ff` — hover for accent surfaces
- **Starlight Press** `#2ba8ea` — active/pressed
- **Starlight Soft** `rgba(76, 194, 255, 0.10)` — selected-row wash, tags
- **Nebula Violet** `#8b7cf6` — gradient partner only (orbit stroke, reward glow); never a flat UI fill

### Surfaces (dark, default)
- **Background Base** `#0a0c12` — window background
- **Background Raised** `#10131c` — panels, cards
- **Background Overlay** `#171c28` — nested surfaces, inputs, hover rows
- **Background Sunken** `#070910` — wells, progress track, telemetry block
- **Border Hairline** `rgba(150, 180, 255, 0.09)` — panel borders, dividers
- **Border Default** `#283043` — controls, inputs
- **Edge Highlight** `rgba(255, 255, 255, 0.045)` — 1px inset top edge on panels

### Text
- **Header Primary** `#eef2fa` — page/brand headings, primary row text
- **Text Normal** `#c3cad9` — body
- **Text Muted** `#8a93a6` — meta, timestamps, hints
- **Text Disabled** `#5b6376` — inactive states
- **Text Link** `#4cc2ff` — links (shares the accent role)

### Status & Semantic
- **Success** `#3ecf8e` — connected, completed
- **Warning** `#f0b232` — idle, checking, degraded
- **Danger** `#f25757` — errors, destructive
- **Reward Gold** `#f0c649` — orb amounts, reward chips, celebration

## 3. Typography Rules

### Font Families
- **UI / Body / Headings**: system stack
  `-apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue", Arial, sans-serif`
- **Numbers / Process names**: monospace stack
  `"SF Mono", ui-monospace, "Cascadia Code", Consolas, monospace`

### Hierarchy

| Role | Size | Weight | Line Height | Notes |
|------|------|--------|-------------|-------|
| Brand / Page title | 16px | 600 | 1.25 | Header left, title-case "Astral" |
| Orbit numeral | 32px | 500 | 1.0 | Mono, ring center |
| Body / Quest name | 14px | 550→500 | 1.35 | Primary row text |
| Meta / caption | 12px | 400 | 1.4 | Muted |
| Micro label | 11px | 600 | 1.2 | Uppercase, 0.10em tracking, muted |
| Mono value | 13px | 500 | 1.2 | Countdown, telemetry values |

### Principles
- Hierarchy comes from weight, color, and case — no invented sizes below 11px.
- Micro-caps are for section labels only, never for body copy or buttons.
- Numbers always mono so they stop jumping width during countdown.

## 4. Component Stylings

### Panels
- Background `#10131c`, border 1px hairline, radius 12px
- Starlit edge: `inset 0 1px 0 rgba(255,255,255,0.045)`
- Padding 16–20px; internal gap 12–16px

### Buttons
- **Primary**: bg `#4cc2ff`, text `#071018` (dark on bright accent), radius 8px, padding 8px 16px, hover `#6fd0ff`, active `#2ba8ea`
- **Secondary**: bg `#1c2333`, border 1px `#283043`, text `#c3cad9`, hover bg `#242d42`
- **Ghost**: transparent, text `#c3cad9`, hover bg `rgba(150,180,255,0.07)`
- **Danger**: bg `#f25757`, text `#160708`, hover `#ff6b6b`
- Disabled: bg `rgba(150,180,255,0.06)`, text `#5b6376`, no pointer events
- Focus: 2px outline `#4cc2ff`, offset 2px

### Inputs
- Background `#171c28`, border 1px `#283043`, radius 8px, padding 8px 12px
- Placeholder `#5b6376`; focus border `#4cc2ff` + 2px ring at 35% alpha

### Quest Rows
- Background `#121623`; hover `rgba(150,180,255,0.05)`; radius 10px; border 1px hairline
- Selected/running: border `#4cc2ff`, bg `rgba(76,194,255,0.07)`
- Left: name (14px/500 header) + meta (12px muted). Right: reward in gold + mono progress label
- Verified tick: 12px check in success color next to the name when `catalog_verified`
- Progress bar: 3px track `rgba(150,180,255,0.10)`, gradient fill cyan→violet, radius full

### Status Pill
- Connected: bg `rgba(62,207,142,0.10)`, border `rgba(62,207,142,0.32)`, dot `#3ecf8e`
- Checking: bg `rgba(240,178,50,0.10)`, border `rgba(240,178,50,0.30)`, dot `#f0b232`
- Disconnected: bg `rgba(138,147,166,0.10)`, border `rgba(138,147,166,0.30)`, dot `#8a93a6`
- Dot 8px, no glow; text 12px 500

### Toggle Switch (demo mode)
- 32×18px track, radius full; off: `#1c2333` + border `#283043`; on: `#4cc2ff`
- 14px knob, white, 150ms slide; label 12px 500

## 5. Spacing & Layout

- **Base unit**: 4px. Scale: 4, 8, 12, 16, 20, 24, 32, 40.
- Two-column workspace (≥ 880px): quest list (flex 3) + session panel (fixed 320px)
- Below 880px: single column, session panel after list
- Window padding 16px; header 52px with bottom hairline
- Orbit ring 148px on desktop, 124px on small windows

## 6. Motion

- Hover: 120ms `ease-out`
- Orbit stroke: 300ms `ease-out`; comet head carries a soft 6px glow (opacity 0.5) while running
- Status transitions: 200ms fade
- Reward overlay: 220ms pop with gold orb-rise; disabled on `prefers-reduced-motion`
- No looping, pulsing, or marquee animations outside the reward overlay

## 7. Usage Guardrails

- Cyan is for *action and progress* only; never decorate headings, badges, or
  idle UI with it. Violet never appears as a flat fill.
- Gold appears only where orbs/rewards are; if the element is not about a
  reward, it is not gold.
- Color never carries information alone — every status has text or an icon.
- No marketing numerals, no flavor names, no claims the UI does not perform.
- Every interactive element must be a real button/input; clickable divs are
  banned. Keyboard focus must be visible.
- Numbers and durations render in the mono stack.
- Preserve the dark shell and compact density; a light theme or balloon
  radii break the instrument-panel feel.
