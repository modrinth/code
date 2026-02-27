# Architecture

The shared UI package used by both `apps/frontend` (Nuxt 3) and `apps/app-frontend` (Vue 3 + Tauri). Components here must be platform-agnostic — use dependency injection for platform-specific behavior.

## Folder Structure

```
src/
├── components/       # Vue components organized by feature domain
├── composables/      # Vue 3 composition API hooks
├── providers/        # Dependency injection contexts (createContext pattern)
├── utils/            # Utility functions and constants
├── pages/            # Cross platform page components (used in both app-frontend and frontend)
├── locales/          # 34 language locale files (FormatJS)
├── styles/           # Tailwind CSS utilities
└── stories/          # Storybook story files
```

Each subdirectory under `components/` has an `index.ts` barrel file. All public API is re-exported from the root `index.ts`.

# Code Guidelines

### Tailwind Configuration

All frontend packages share a Tailwind preset at `packages/tooling-config/tailwind/tailwind-preset.ts`. This package's `tailwind.config.ts` extends it:

```ts
import preset from '@modrinth/tooling-config/tailwind/tailwind-preset.ts'
```

CSS custom properties are defined in `packages/assets/styles/variables.scss` with light, dark, and OLED theme variants.

### Color Usage Rules

**Use `surface-*` variables for backgrounds — never aliased `bg-*` color variables:**

| Token            | Usage                                     |
| ---------------- | ----------------------------------------- |
| `bg-surface-1`   | Deepest background layer                  |
| `bg-surface-1.5` | Odd row background (tables)               |
| `bg-surface-2`   | Even row background, secondary panels     |
| `bg-surface-3`   | Headers, floating bar backgrounds, inputs |
| `bg-surface-4`   | Cards, elevated surfaces                  |
| `bg-surface-5`   | Borders, dividers                         |

**For text colors:**

| Class            | Usage                            |
| ---------------- | -------------------------------- |
| `text-contrast`  | Primary headings                 |
| `text-primary`   | Default body text                |
| `text-secondary` | Reduced emphasis, secondary info |

**Brand and semantic colors** not all exposed as Figma variables — refer to `packages/assets/styles/variables.scss` for the full set:

- `bg-{color}`, `text-{color}`  etc. — Primary brand colors
- `bg-{color}-highlight` — 25% opacity semantic highlights

**Color palette** (each with shades 50–950): red, orange, green, blue, purple, gray. Platform-specific colors also exist (fabric, forge, quilt, neoforge, etc.).

## Dependency Injection

This package defines the DI layer using `createContext` from `src/providers/index.ts`. See the `dependency-injection` skill (`.claude/skills/dependency-injection/SKILL.md`) for full documentation.

Key providers exported from this package:
- `provideModrinthClient` / `injectModrinthClient` — API client
- `provideNotificationManager` / `injectNotificationManager` — Notifications
