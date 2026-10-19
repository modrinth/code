<picture>
	<source media="(prefers-color-scheme: light)" srcset="../../.github/assets/ui_cover_light.png">
	<img alt="Modrinth UI Cover" src="../../.github/assets/ui_cover.png">
</picture>

# @modrinth/ui

The shared Vue 3 component library used by both the [Modrinth Website](../../apps/frontend) and the [Modrinth App](../../apps/app-frontend). Components are platform-agnostic, with platform-specific behavior provided through dependency injection.

## Structure

```
src/
├── components/   # Vue components, grouped by feature
├── composables/  # Composition API hooks
├── layouts/      # Page layouts shared between the website and app
├── providers/    # Dependency injection contexts
├── utils/        # Utility functions and constants
├── locales/      # Translations
├── styles/       # Tailwind CSS utilities
└── stories/      # Storybook stories
```

Everything public is re-exported from [`index.ts`](./index.ts).

## Development

Browse and develop components in isolation with Storybook, from the repository root:

```bash
pnpm storybook
```

## License

Licensed under the GNU General Public License v3. Modrinth branding is excluded, see [COPYING.md](./COPYING.md) for details.
