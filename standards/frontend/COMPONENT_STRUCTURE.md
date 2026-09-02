# Component Structure

## Component Folders

Give each complex component its own folder:

```
components/
└── analytics-chart/
	├── index.vue
	├── analytics-chart-header.vue
	├── analytics-chart-plot.vue
	├── analytics-chart-data.ts
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
├── analytics-chart-header.vue
├── analytics-chart-plot.vue
├── analytics-chart-tooltip.vue
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

Use clear names that show the relation between each subcomponent and its main component:

```
analytics-chart/
├── index.vue
├── analytics-chart-header.vue
└── analytics-chart-plot.vue
```

Do not use names that make a local component look like a public component:

```
analytics-chart/
├── index.vue
├── events.vue
└── header.vue
```

Add the `analytics-chart-` prefix to local filenames. This prefix shows the relation in search results, editor tabs, and imports.

## Nesting

Use one nesting level in most component folders.

Use this structure:

```
analytics-chart/
├── index.vue
├── analytics-chart-header.vue
├── analytics-chart-plot.vue
├── use-chart-hover-state.ts
└── use-chart-selection.ts
```

Do not use this structure unless a local area needs its own module boundary:

```
analytics-chart/
├── index.vue
├── header/
│	└── index.vue
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

Move a component into a folder when it gets local helpers, composables, or subcomponents.

## Public and Local Components

Use only the main `index.vue` as the public entry point. Treat the other folder files as implementation details.

If another component imports a local subcomponent, use one of these solutions:

- Move the subcomponent into its own component folder.
- Move the subcomponent to the nearest shared component area when it is reusable.
- Keep it local and pass behavior through the main component.

Use the last solution when an external import exposes implementation details.
