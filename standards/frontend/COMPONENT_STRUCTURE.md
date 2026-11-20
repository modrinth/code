# Component Structure

For `app-frontend`, choose the feature or shell folder using [Application Structure](app-frontend/APP_FRONTEND_STRUCTURE.md) before organising the component's own files. Keep its supporting code nearby, with larger feature behavior outside the component folder. [App Composables](app-frontend/APP_FRONTEND_COMPOSABLES.md) explains when it helps to separate reactive behavior.

These app-specific placement rules do not change the website or shared UI folder structure.

## File Names

Name every standalone component file in PascalCase, matching the component name: `EditServerModal.vue`, `AvatarStack.vue`, `WorldItem.vue`. A standalone component is one that is not split up into a folder of its own. This applies wherever the file lives, including a `components/` folder inside a larger component:

```
quickplay-page/
├── index.vue
├── types.ts
└── components/
    ├── AddServerModal.vue
    ├── EditServerModal.vue
    └── QuickplayToolbar.vue
```

Use lowercase kebab-case `.vue` names only for files that are parts of a component folder: the folder's `index.vue` entry point and the role-named pieces beside it, such as `header.vue` or `plot.vue` inside `analytics-chart/`. These files are implementation details of the folder's component, so the folder supplies the public name.

Do not name a standalone component in kebab case. `edit-server-modal.vue` is incorrect; use `EditServerModal.vue`.

Folders, composables, and other TypeScript files always use kebab case: `analytics-chart/`, `use-chart-hover-state.ts`, `types.ts`.

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

Use short lowercase names that describe each local component's role. The parent folder already supplies the component name, so `header.vue` is enough inside `analytics-chart/`:

```
analytics-chart/
├── index.vue
├── header.vue
└── plot.vue
```

Use a more specific name when the role is unclear, such as `date-filter.vue` or `status-filter.vue`. There is no need to repeat the parent name in every filename.

This lowercase form is only for pieces of a component folder. A component in a grouping folder such as `components/` is standalone and uses PascalCase, as described in [File Names](#file-names).

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

Keep small leaf components in single PascalCase `.vue` files:

```
components/
├── AvatarStack.vue
├── EmptyState.vue
└── ProjectStatusPill.vue
```

Move a component into a folder when it gets local helpers, composables, types, or subcomponents. The folder takes the kebab-case name and the component becomes its `index.vue`. If only `index.vue` remains in a folder, move it back to a PascalCase file in the parent folder and update its imports. For example, use `app-shell/RouteOutlet.vue`, not `app-shell/route-outlet/index.vue`. This rule also applies to large components that have no supporting files.

## Public and Local Components

For a component with its own folder, use the main `index.vue` as its public entry point. Treat the supporting files as implementation details. A standalone component uses its PascalCase `.vue` file as its entry point; it does not need a folder to be public.

If another component imports a local subcomponent, use one of these solutions:

- Promote the subcomponent to a standalone PascalCase `.vue` file, or give it its own component folder if it has supporting files.
- Move the subcomponent to the nearest shared component area when it is reusable.
- Keep it local and pass behavior through the main component.

Use the last solution when an external import exposes implementation details.
