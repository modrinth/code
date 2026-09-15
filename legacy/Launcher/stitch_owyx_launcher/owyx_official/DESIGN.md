---
name: Owyx Official
colors:
  surface: '#131319'
  surface-dim: '#131319'
  surface-bright: '#39383f'
  surface-container-lowest: '#0e0e14'
  surface-container-low: '#1b1b21'
  surface-container: '#1f1f25'
  surface-container-high: '#2a2930'
  surface-container-highest: '#35343b'
  on-surface: '#e4e1ea'
  on-surface-variant: '#bac9cc'
  inverse-surface: '#e4e1ea'
  inverse-on-surface: '#303037'
  outline: '#849396'
  outline-variant: '#3b494c'
  surface-tint: '#00daf3'
  primary: '#c3f5ff'
  on-primary: '#041018'
  primary-container: '#00e5ff'
  on-primary-container: '#00626e'
  inverse-primary: '#006875'
  secondary: '#d0bcff'
  on-secondary: '#3c0091'
  secondary-container: '#571bc1'
  on-secondary-container: '#c4abff'
  tertiary: '#ffeac0'
  on-tertiary: '#3e2e00'
  tertiary-container: '#fec931'
  on-tertiary-container: '#6f5500'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#9cf0ff'
  primary-fixed-dim: '#00daf3'
  on-primary-fixed: '#001f24'
  on-primary-fixed-variant: '#004f58'
  secondary-fixed: '#e9ddff'
  secondary-fixed-dim: '#d0bcff'
  on-secondary-fixed: '#23005c'
  on-secondary-fixed-variant: '#5516be'
  tertiary-fixed: '#ffdf96'
  tertiary-fixed-dim: '#f3bf26'
  on-tertiary-fixed: '#251a00'
  on-tertiary-fixed-variant: '#594400'
  background: '#050508'
  on-background: '#e4e1ea'
  surface-variant: '#35343b'
  surface-raised: '#14141c'
  border: '#2a2b30'
  text-primary: '#f3f4f6'
  text-muted: '#9aa0a8'
typography:
  display-lg:
    fontFamily: DM Sans
    fontSize: 26px
    fontWeight: '700'
    lineHeight: 30px
    letterSpacing: -0.03em
  headline-md:
    fontFamily: DM Sans
    fontSize: 20px
    fontWeight: '700'
    lineHeight: 24px
    letterSpacing: -0.02em
  title-sm:
    fontFamily: DM Sans
    fontSize: 16px
    fontWeight: '600'
    lineHeight: 20px
  body-md:
    fontFamily: DM Sans
    fontSize: 15px
    fontWeight: '500'
    lineHeight: 22px
  body-sm:
    fontFamily: DM Sans
    fontSize: 13px
    fontWeight: '500'
    lineHeight: 18px
  label-xs:
    fontFamily: DM Sans
    fontSize: 12px
    fontWeight: '600'
    lineHeight: 14px
    letterSpacing: 0.04em
  brand-nav:
    fontFamily: DM Sans
    fontSize: 15px
    fontWeight: '700'
    lineHeight: 15px
    letterSpacing: -0.02em
rounded:
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  rail-width: 64px
  topbar-height: 48px
  page-padding: 24px
  gap-xs: 4px
  gap-sm: 8px
  gap-md: 12px
  gap-lg: 16px
  gap-xl: 24px
---

## Brand & Style

The design system moves into its official Phase 2 identity, transitioning from a generic utility to a specialized, high-fidelity gaming environment. The brand personality is technical, precise, and immersive, targeting a power-user demographic that values speed and stability.

The design style is **Corporate / Modern** with a **High-Contrast** edge. It utilizes a deep-space palette to create a focused "low-light" environment, where high-chroma cyan accents act as functional beacons for primary actions. The aesthetic is clean and structured, favoring data density and clear navigation over decorative elements. A signature "segmented ring" visual language is applied to icons and logos to reinforce the unique Owyx identity. All previous green accents are strictly removed in favor of a sophisticated Cyan and Violet pairing.

## Colors

The palette is anchored by a near-black foundation that minimizes eye strain and maximizes the impact of brand accents.

- **Primary (Cyan):** Reserved for high-priority interactive elements like "Play," "New Instance," and "Create." It represents action and energy.
- **Secondary (Violet):** Used primarily as a "wash" or background tint for active states and selection indicators, providing a subtle layer of depth.
- **Neutral Stack:**
    - **Background:** The base app canvas.
    - **Surface:** Standard component backgrounds.
    - **Raised:** Elevated elements such as cards and inputs.
    - **Border:** Subtle structural separation.
- **Text:** High-contrast off-white for readability, with a cool-gray muted tone for secondary metadata and inactive labels.

## Typography

DM Sans is utilized across the entire system to maintain a modern, geometric clarity. The hierarchy is designed for high information density typical of desktop launchers. 

Larger display styles use tighter tracking and leading to feel "locked-in" and architectural. Body text is prioritized for legibility with slightly more generous line height. Small labels and metadata use increased letter spacing and semi-bold weights to ensure they remain scannable against dark backgrounds.

## Layout & Spacing

The system follows a **Fixed Grid** shell model optimized for desktop application windows (1280px+).

- **Navigation Rail:** A permanent 64px vertical sidebar on the left.
- **Top Bar:** A 48px horizontal utility bar containing branding and global status.
- **Content Area:** A scrollable stage with consistent 24px internal padding.

**Navigation Rail Hierarchy:**
1. **Logo:** Segmented ring 'O' with purple-to-cyan glow.
2. **Navigation:** Home, Library, Private Servers (Discover is removed).
3. **Activity:** Recent Instances (circular avatars, collapses when empty).
4. **Action:** '+' Create Instance.
5. **Spacer:** Flexible vertical expansion.
6. **Utility:** Settings (anchored to the bottom).

## Elevation & Depth

The design system uses **Tonal Layers** to establish hierarchy, avoiding traditional drop shadows in favor of surface-based depth.

- **Level 0 (Background):** Pure #050508 foundation.
- **Level 1 (Surface):** Default panel state (#0c0c12).
- **Level 2 (Raised):** Interactive components like cards and inputs (#14141c).
- **Separation:** 1px borders (#2a2b30) are used for all structural definition. 
- **Modals:** Use a dark backdrop scrim with a subtle background blur to focus attention. 
- **Active States:** Indicated by a #8b5cf6 (Violet) wash at low opacity, creating a distinct "lit" effect without cluttering the UI.

## Shapes

The shape language is a mix of geometric precision and organic softness. 

- **Circular (Full):** Used for navigation icons, status pills, and avatars to create a friendly, "tappable" feel within the tech-heavy environment.
- **Rounded (0.5rem - 1.5rem):** Used for containers, cards, and input fields to soften the "pro-tool" aesthetic and improve visual flow.
- **Segmented Ring:** The core brand motif, applied to the logo and potentially custom loading indicators.

## Components

### Buttons
- **Primary (Play/Create):** Solid Cyan (#00e5ff) background with Dark (#041018) text. 10px corner radius. High-impact bold weight.
- **Ghost/Secondary:** Raised (#14141c) background with a 1px border. For secondary navigation or utility actions.
- **Rail Button:** 48x48 circular hit targets. Active state: Violet (#8b5cf6) background wash with a Cyan (#00e5ff) icon.

### Input Fields
- **Standard:** Raised background, 10px radius, 1px border. Focus state should use a Cyan border or a subtle Violet glow.

### Cards & Lists
- **Instance Cards:** Surface background, 14px radius. Contains a prominent Play button on hover/active.
- **Instance Rows:** Horizontal layout with a clear trailing action button.

### Navigation Elements
- **Tabs/Chips:** Pill-shaped (fully rounded). Inactive uses Raised background; Active uses Violet wash with Cyan text.
- **Top Bar Status:** A pill-shaped component on the right side indicating launch progress or active session status.

### Interaction Logic
- **Hover:** Subtle lightening of the background color or border.
- **Active:** Violet wash application for selection visibility.
- **Accessibility:** All Cyan elements must maintain high contrast against the dark background; avoid using Cyan for body text.