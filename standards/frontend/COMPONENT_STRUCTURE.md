# Component Structure

For `app-frontend`, choose the feature or shell folder using [Application Structure](app-frontend/APP_FRONTEND_STRUCTURE.md) before organising the component's own files. Keep its supporting code nearby, with larger feature behavior outside the component folder. [App Composables](app-frontend/APP_FRONTEND_COMPOSABLES.md) explains when it helps to separate reactive behavior.

These app-specific placement rules do not change the website or shared UI folder structure.

## Component Folders

Give a component its own folder when it has local helpers, composables, types, or subcomponents. Do not create a folder containing only `index.vue`; keep that component as a named `.vue` file in its parent folder.

```
components/
└── analytics-chart/
	├── index.vue
	├── header.vue
	├── plot.vue
	├── data.ts
	└── use-analytics-chart.ts
```

Use the public component name in kebab case for the folder name. Use `index.vue` for the main component.

This structure keeps imports short:

```ts
import AnalyticsChart from '@/components/analytics-chart/index.vue'
```

You can import the folder if the local resolver supports directory indexes:

```ts
import AnalyticsChart from '@/components/analytics-chart/'
```

Use the explicit `index.vue` import if TypeScript cannot resolve the directory import.

## Local Implementation Files

Keep files for only one component in that component's folder:

```
analytics-chart/
├── index.vue
├── header.vue
├── plot.vue
├── tooltip.vue
├── chart-ranges.ts
└── use-chart-hover-state.ts
```

Use local files for these items:

- Small subcomponents that only the main component uses.
- Local composables that only the component folder uses.
- Helpers that divide a large `<script setup>` block.
- Types for local component state or props.

This structure prevents large script blocks that are difficult to review.

## Local Subcomponent Names

Use short names that describe each local component's role. The parent folder already supplies the component name, so `header.vue` is enough inside `analytics-chart/`:

```
analytics-chart/
├── index.vue
├── header.vue
└── plot.vue
```

Use a more specific name when the role is unclear, such as `date-filter.vue` or `status-filter.vue`. There is no need to repeat the parent name in every filename.

## Nesting

Use one nesting level in most component folders.

Use this structure:

```
analytics-chart/
├── index.vue
├── header.vue
├── plot.vue
├── use-chart-hover-state.ts
└── use-chart-selection.ts
```

Use a subfolder only when a local area needs its own module boundary and has supporting files:

```
analytics-chart/
├── index.vue
├── header.vue
└── plot/
	├── index.vue
	└── use-plot-state.ts
```

Use subfolders when they reduce real complexity. Do not make a folder for each small subcomponent.

Deep nesting makes the file tree difficult to scan. It also causes duplicate names without clearer ownership.

## Small Components

Keep small leaf components in single `.vue` files:

```
components/
├── avatar-stack.vue
├── empty-state.vue
└── project-status-pill.vue
```

Move a component into a folder when it gets local helpers, composables, types, or subcomponents. If only `index.vue` remains in a folder, move it back to a named file in the parent folder and update its imports. For example, use `app-shell/route-outlet.vue`, not `app-shell/route-outlet/index.vue`. This rule also applies to large components that have no supporting files.

## Public and Local Components

For a component with its own folder, use the main `index.vue` as its public entry point. Treat the supporting files as implementation details. A standalone component uses its named `.vue` file as its entry point; it does not need a folder to be public.

If another component imports a local subcomponent, use one of these solutions:

- Promote the subcomponent to a standalone named `.vue` file, or give it its own component folder if it has supporting files.
- Move the subcomponent to the nearest shared component area when it is reusable.
- Keep it local and pass behavior through the main component.

Use the last solution when an external import exposes implementation details.
