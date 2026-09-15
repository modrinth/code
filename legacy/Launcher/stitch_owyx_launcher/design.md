---
version: alpha
name: Owyx Launcher
description: >-
  Dark desktop Minecraft launcher UI (Windows / Tauri WebView). Phase 1 mirrors
  Modrinth App information architecture and green-accent dark chrome; Phase 2
  will rebrand to Owyx individuality. Product is a native-feeling desktop app,
  not a marketing website — design for ~1280×800 and larger desktop windows.
colors:
  bg: "#09090b"
  surface: "#121214"
  surface-raised: "#1a1b1e"
  rail: "#0b0b0d"
  border: "#2a2b30"
  text: "#f3f4f6"
  text-muted: "#9aa0a8"
  primary: "#1bd96a"
  on-primary: "#042313"
  danger: "#ff5c6c"
  danger-surface: "#3a1519"
  ok: "#7dcea0"
  overlay: "rgba(0,0,0,0.58)"
  focus-ring: "#1bd96a"
typography:
  display:
    fontFamily: DM Sans
    fontSize: 26px
    fontWeight: 700
    lineHeight: 1.15
    letterSpacing: -0.03em
  headline:
    fontFamily: DM Sans
    fontSize: 20px
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: -0.02em
  title:
    fontFamily: DM Sans
    fontSize: 16px
    fontWeight: 600
    lineHeight: 1.3
  body:
    fontFamily: DM Sans
    fontSize: 15px
    fontWeight: 500
    lineHeight: 1.45
  body-sm:
    fontFamily: DM Sans
    fontSize: 13px
    fontWeight: 500
    lineHeight: 1.4
  label:
    fontFamily: DM Sans
    fontSize: 12px
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: 0.04em
  brand:
    fontFamily: DM Sans
    fontSize: 15px
    fontWeight: 700
    lineHeight: 1
    letterSpacing: -0.02em
rounded:
  sm: 8px
  md: 10px
  lg: 14px
  xl: 16px
  full: 9999px
spacing:
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 24px
  xxl: 32px
  rail-width: 64px
  topbar-height: 48px
  page-padding: 24px
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    rounded: "{rounded.md}"
    padding: 10px 14px
    height: 40px
  button-ghost:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    rounded: "{rounded.md}"
    padding: 10px 14px
  button-danger:
    backgroundColor: "{colors.danger-surface}"
    textColor: "#ff9aa5"
    rounded: "{rounded.md}"
  rail-button:
    backgroundColor: transparent
    textColor: "{colors.text-muted}"
    rounded: "{rounded.full}"
    width: 48px
    height: 48px
  rail-button-active:
    backgroundColor: "color-mix(in srgb, #1bd96a 16%, transparent)"
    textColor: "{colors.primary}"
  input:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    rounded: "{rounded.md}"
    padding: 11px 12px
  chip:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    rounded: "{rounded.full}"
    padding: 9px 12px
  chip-selected:
    backgroundColor: "color-mix(in srgb, #1bd96a 12%, #1a1b1e)"
    textColor: "{colors.primary}"
  tab:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.full}"
    padding: 8px 14px
  tab-active:
    backgroundColor: "color-mix(in srgb, #1bd96a 18%, #1a1b1e)"
    textColor: "{colors.primary}"
  card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    rounded: "{rounded.lg}"
    padding: 18px
  modal:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.xl}"
    padding: 18px
  switch-on:
    backgroundColor: "{colors.primary}"
  switch-off:
    backgroundColor: "#3a3b40"
  status-pill:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.full}"
    padding: 6px 12px
---

# Owyx Launcher — DESIGN.md

## Overview

**Owyx** is a custom **desktop Minecraft launcher** (Windows first, Tauri + WebView). The UI must feel like a **native desktop app**, not a marketing landing page and not a mobile app.

**Phase 1 visual direction:** close to **Modrinth App** — near-black chrome, single bright green accent for primary actions and selection, circular left rail icons, pill tabs, dense but readable lists. This is intentional: we are building a working Modrinth-like reference before an Owyx rebrand.

**Phase 2 (later, do not invent yet in generated screens unless asked):** unique Owyx branding, colors, logos, private-server emphasis, custom login site — may change layout.

**Personality:** quiet, technical, gamer-tool. Confident green CTAs. Minimal decoration. No purple gradients, no neon glow spam, no emoji as UI chrome, no newspaper/serif editorial look.

**Audience:** Minecraft players installing and launching instances (Vanilla / Fabric / Quilt / Forge / NeoForge), managing mods later, and (later) joining author private servers.

**Language in UI for Phase 1 mockups:** English labels matching Modrinth patterns (`Home`, `Library`, `New instance`, `Play`, `Settings`). Product name in chrome: lowercase **owyx**.

**Window:** frameless-feeling content under a thin app top bar (brand + breadcrumb + status pill). Design **desktop Web** layouts at **1280×800** and **1440×900**. Always show the **left rail**.

## Colors

Near-black foundation with one Modrinth-like green accent.

- **Background (`#09090b`):** App canvas. Optional very subtle green/cool radial wash — never loud.
- **Surface (`#121214`) / Raised (`#1a1b1e`):** Panels, cards, inputs, rail.
- **Border (`#2a2b30`):** 1px separators and control outlines.
- **Text (`#f3f4f6`) / Muted (`#9aa0a8`):** Primary vs secondary copy.
- **Primary / Accent (`#1bd96a`):** Play, New instance, selected tabs/chips, active rail, toggles on. Ink on green: `#042313`.
- **Danger (`#ff5c6c`):** Delete only — never for primary navigation.
- **OK (`#7dcea0`):** Success / valid Java path checkmarks.

One accent color only. Do not introduce purple, pink, or rainbow accents.

## Typography

**DM Sans** for everything (UI sans). No serif. No Inter-as-default marketing stack feel — DM Sans is the brand type for Phase 1.

- Page titles: ~26px / 700 / tight tracking.
- Section titles: ~16–20px / 700.
- Body / inputs: 14–15px / 500.
- Meta lines under instance names: 12–13px muted.
- Rail has no text labels — icons + tooltips only.

## Layout

**Shell grid (always):**

1. **Left rail** — fixed **64px** (`4rem`) width. Vertical stack.
2. **Stage** — topbar 48px + scrollable main content with ~24px padding.

**Left rail order (top → bottom), Modrinth-shaped:**

1. Small circular **O** logo mark (green gradient).
2. Nav icons (circular 48×48 hit targets): **Home**, **Discover**, **Library**, **Private Servers**.
3. **Recent instances** — up to ~6 circular avatar buttons (initial letter). Expands with content; **collapses when empty** (no empty gap).
4. **+** create instance (same circular control style).
5. **Flex spacer** pushing settings down.
6. **Settings** gear at the very bottom.

Important: **+ is NOT next to Settings by default.** Settings alone sits at the bottom. + sits directly under recent (or under nav when recent is empty).

**Top bar:** left — `owyx` brand + optional back/forward (can be disabled) + breadcrumb (` / Home`); right — status pill (`No active instances` / install progress / play status).

**Home:** two columns on desktop — main (Welcome + Jump back in list) + right sticky column (~300px) for **Account** (nick, skin placeholder, Microsoft login disabled) and **News**.

**Library:** header row with pill tabs (`All instances` active; Modpacks/Servers/Custom disabled) + green **+ New instance**; search + sort/group; instance rows OR centered empty state.

**Instance detail:** hero (icon, name, MC/loader meta, **Play**, gear, ⋮ menu) + pill tabs: Content | Files | Worlds | Logs | Share.

**Settings (launcher):** split view — left settings nav (~220px), right content card (Java installations first).

**Modals:** centered card on dimmed blurred overlay; max width ~560px for create; instance settings can be full-page split instead of modal (both are valid — prefer **modal for create**, **page for instance settings** and **app settings**).

Spacing rhythm: 4 / 8 / 12 / 16 / 24 px.

## Elevation & Depth

Mostly **flat tonal layers**: background → surface → raised. Borders over drop shadows.

- Soft shadow only on **dropdown menus** and **modals** (`0 12–24px` dark translucent).
- Modal backdrop: dark scrim + light blur.
- Active rail: filled tint, **no** left accent bar required (Modrinth-style selected fill).
- Avoid multi-layer neon glows.

## Shapes

- Rail buttons / recent avatars / tabs / chips / status pill: **fully rounded** (`9999px`).
- Cards, inputs, primary buttons, modal: **10–16px** radius.
- Instance icons: ~10–14px radius squares.
- Consistency: pills for filters/tabs; rounded-rect for forms and cards.

## Components

### Buttons

- **Primary:** solid `#1bd96a`, dark green text, 10px radius, bold. Used for Play, New instance, Create, Find projects (when enabled).
- **Ghost:** raised fill + border. Secondary actions (Detect, Add files, Cancel).
- **Danger:** dark red surface — Delete instance only.
- **Icon button:** 40×40, raised, border — settings, overflow, close.

### Rail / NavButton

Circular, icon-only, 48×48. Hover: raised fill. Active: green tint + green icon.

### Inputs & selects

Full-width-friendly, raised background, 10px radius, muted placeholder. Selects with custom chevron.

### Chips & tabs

Pill shaped. Selected = green text + light green wash. Used for loader pick (Fabric/Forge/…) and library/instance filters.

### Lists

- **Jump-back / instance rows:** horizontal bar — leading icon, title + meta, trailing Play (home) or chevron.
- **Content table:** columns Project | Version | Actions (toggle, trash).
- **Java rows:** path + green check.

### Switch

Track 40×22; on = primary green; off = gray.

### Modal (Create instance)

Fields top to bottom: Title → Platform chips → Game version select → Loader version select (hidden for Vanilla) → Snapshots checkbox → status line → Create / Cancel.

### Instance settings page

Left nav: General | Installation | Window | Java & memory | Launch commands. Right pane forms; General includes rename, group, update channel pills (Release/Beta/Alpha), Duplicate, Delete, Back.

### Empty states

Centered, muted, one short headline + one sentence + optional primary CTA. Library empty: **No instances found** + Create new instance.

## Do's and Don'ts

**Do**

- Keep the left rail visible on every primary screen.
- Use green only for primary CTA and selection states.
- Design desktop density (Modrinth App), not mobile-first cards stacked for phones.
- Show install/play status in the top-right pill when relevant.
- Match Modrinth IA: Home / Discover / Library / Servers / + / Settings.
- Prefer icon+label clarity; English UI strings for Phase 1 mocks.

**Don't**

- Don’t put + and Settings in the same bottom cluster as the default layout.
- Don’t use purple/indigo AI-default themes, cream+serif terracotta, or broadsheet newspaper layouts.
- Don’t invent a marketing landing page; this is an **app shell**.
- Don’t overload the first viewport with stats strips, badge clutter, or floating promo stickers on heroes.
- Don’t hide Play behind nested menus — Play stays prominent on instance and jump-back rows.
- Don’t redesign Owyx brand colors in Phase 1 generations — stick to this green/black system until Phase 2 brief.
