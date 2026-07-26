# Component Structure

## Component folders

Prefer giving non-trivial components their own folder:

```
components/
└── analytics-chart/
	├── index.vue
	├── analytics-chart-header.vue
	├── analytics-chart-plot.vue
	├── analytics-chart-data.ts
	└── use-analytics-chart.ts
```

The folder name should match the public component name in kebab case. The main component in that folder should be `index.vue`.

This keeps imports short:

```ts
import AnalyticsChart from '@/components/analytics-chart/index.vue'
```

If the local resolver supports directory indexes, importing the folder is also fine:

```ts
import AnalyticsChart from '@/components/analytics-chart/'
```

Use the explicit `index.vue` import when the TypeScript setup cannot resolve the directory import reliably.

## Local implementation files

Keep files that only exist to support one component inside that component's folder:

```
analytics-chart/
├── index.vue
├── analytics-chart-header.vue
├── analytics-chart-plot.vue
├── analytics-chart-tooltip.vue
├── chart-ranges.ts
└── use-chart-hover-state.ts
```

Good candidates for local files:

- Small subcomponents used only by the main component
- Local composables used only by the main component or its local subcomponents
- Helpers that split up a large `<script setup>` block
- Types that describe local component state or props

This is preferred over allowing a single component file to grow into a large, hard-to-review script block.

## Naming local subcomponents

Local subcomponents should still have clear names that explain their relationship to the main component:

```
analytics-chart/
├── index.vue
├── analytics-chart-header.vue
└── analytics-chart-plot.vue
```

Avoid vague names that make a local component look like a standalone public component:

```
analytics-chart/
├── index.vue
├── events.vue
└── header.vue
```

If a file is local to `analytics-chart`, prefixing it with `analytics-chart-` makes that relationship clear when it appears in search results, editor tabs, and imports.

## Nesting

One level of nesting is usually enough.

Prefer this:

```
analytics-chart/
├── index.vue
├── analytics-chart-header.vue
├── analytics-chart-plot.vue
├── use-chart-hover-state.ts
└── use-chart-selection.ts
```

Avoid this unless a local area has become large enough to justify its own module boundary:

```
analytics-chart/
├── index.vue
├── header/
│	└── index.vue
└── plot/
	├── index.vue
	└── use-plot-state.ts
```

Subfolders are fine when they reduce real complexity, but do not create a folder for every small subcomponent by default. Deep nesting makes the file tree harder to scan and often adds duplicated names without improving ownership.

## When not to use a folder

Small, leaf components can stay as a single `.vue` file:

```
components/
├── avatar-stack.vue
├── empty-state.vue
└── project-status-pill.vue
```

Move a component into a folder once it grows local helpers, local composables, or local subcomponents.

## Public versus local components

Only the main `index.vue` should be treated as the public entry point for the folder. Other files in the folder are implementation details unless there is a clear reason to import them from outside.

If a local subcomponent starts being imported elsewhere, either:

- Promote it into its own component folder
- Move it to the nearest shared component area if it is genuinely reusable
- Keep it local and pass behavior through the main component if external imports would leak implementation details
