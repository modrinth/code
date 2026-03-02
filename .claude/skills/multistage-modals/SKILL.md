# Multistage Modals

The `MultiStageModal` component (`packages/ui/src/components/base/MultiStageModal.vue`) provides a wizard-like modal with progress tracking, conditional stages, and per-stage button configuration.

## Architecture

A multistage modal has three parts:

1. **Context** — A DI provider that holds all state, business logic, and stage configs
2. **Stage configs** — Data objects describing each stage (title, component, buttons, skip conditions)
3. **Stage components** — Vue components rendered inside the modal, consuming the context

## Building a Multistage Modal

### 1. Define the context

Create a DI provider with all the state your wizard needs. Include the modal ref and stage configs.

```ts
// providers/my-feature/my-modal.ts
import type { ShallowRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'
import type { MultiStageModal, StageConfigInput } from '@modrinth/ui'
import { createContext } from '@modrinth/ui'

export interface MyModalContext {
	// State
	formData: Ref<MyFormData>
	isSubmitting: Ref<boolean>

	// Modal control
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>
	stageConfigs: StageConfigInput<MyModalContext>[]

	// Business logic
	handleSubmit: () => Promise<void>
}

export const [injectMyModalContext, provideMyModalContext] =
	createContext<MyModalContext>('MyModal')

export function createMyModalContext(
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>,
): MyModalContext {
	const formData = ref<MyFormData>({ ... })
	const isSubmitting = ref(false)

	async function handleSubmit() {
		isSubmitting.value = true
		try {
			await saveData(formData.value)
			modal.value?.hide()
		} finally {
			isSubmitting.value = false
		}
	}

	return { formData, isSubmitting, modal, stageConfigs, handleSubmit }
}
```

### 2. Define stage configs

Each stage is a `StageConfigInput<T>` where `T` is your context type. Most fields accept either a static value or a function receiving the context (`MaybeCtxFn<T, R>`).

```ts
// providers/my-feature/stages/details-stage.ts
import { markRaw } from 'vue'
import type { StageConfigInput } from '@modrinth/ui'
import type { MyModalContext } from '../my-modal'
import DetailsStage from './DetailsStage.vue'
import { RightArrowIcon, SaveIcon } from '@modrinth/assets'

export const detailsStageConfig: StageConfigInput<MyModalContext> = {
	id: 'details',
	stageContent: markRaw(DetailsStage),
	title: 'Details',

	// Conditional behavior based on context
	skip: (ctx) => ctx.shouldSkipDetails.value,
	cannotNavigateForward: (ctx) => !ctx.formData.value.name,
	disableClose: (ctx) => ctx.isSubmitting.value,

	leftButtonConfig: (ctx) => ({
		label: 'Cancel',
		onClick: () => ctx.modal.value?.hide(),
	}),

	rightButtonConfig: (ctx) => ({
		label: 'Next',
		icon: RightArrowIcon,
		iconPosition: 'after',
		disabled: !ctx.formData.value.name,
		onClick: () => ctx.modal.value?.nextStage(),
	}),
}
```

**Stage config fields:**

| Field | Type | Purpose |
|-------|------|---------|
| `id` | `string` | Unique stage identifier (used with `setStage()`) |
| `stageContent` | `Component` | Vue component to render (wrap with `markRaw()`) |
| `title` | `MaybeCtxFn<T, string>` | Stage title in breadcrumbs |
| `skip` | `MaybeCtxFn<T, boolean>` | Skip this stage conditionally |
| `nonProgressStage` | `MaybeCtxFn<T, boolean>` | Exclude from progress bar (for edit sub-flows) |
| `hideStageInBreadcrumb` | `MaybeCtxFn<T, boolean>` | Hide from breadcrumb nav |
| `cannotNavigateForward` | `MaybeCtxFn<T, boolean>` | Block forward navigation (validation) |
| `disableClose` | `MaybeCtxFn<T, boolean>` | Disable closing the modal |
| `leftButtonConfig` | `MaybeCtxFn<T, StageButtonConfig \| null>` | Left action button |
| `rightButtonConfig` | `MaybeCtxFn<T, StageButtonConfig \| null>` | Right action button |
| `maxWidth` | `MaybeCtxFn<T, string>` | Per-stage max width (default `560px`) |

**Button config fields:**

| Field | Purpose |
|-------|---------|
| `label` | Button text |
| `icon` | Icon component |
| `iconPosition` | `'before'` or `'after'` |
| `color` | ButtonStyled color prop |
| `disabled` | Disable the button |
| `onClick` | Click handler |

### 3. Create stage components

Stage components inject the context and render their UI:

```vue
<!-- providers/my-feature/stages/DetailsStage.vue -->
<script setup lang="ts">
import { injectMyModalContext } from '../my-modal'

const { formData } = injectMyModalContext()
</script>

<template>
	<div class="flex flex-col gap-4">
		<StyledInput v-model="formData.name" label="Name" />
		<StyledInput v-model="formData.description" label="Description" />
	</div>
</template>
```

### 4. Create the wrapper component

The wrapper provides context and renders `MultiStageModal`:

```vue
<!-- components/MyModalWrapper.vue -->
<script setup lang="ts">
import { shallowRef } from 'vue'
import { MultiStageModal } from '@modrinth/ui'
import { createMyModalContext, provideMyModalContext } from '../providers/my-feature/my-modal'

const modal = shallowRef<InstanceType<typeof MultiStageModal> | null>(null)
const ctx = createMyModalContext(modal)
provideMyModalContext(ctx)

defineExpose({ show: () => modal.value?.show() })
</script>

<template>
	<MultiStageModal ref="modal" :stages="ctx.stageConfigs" :context="ctx" />
</template>
```

## Modal API

`MultiStageModal` exposes via ref:

| Method/Property | Description |
|----------------|-------------|
| `show()` | Open the modal |
| `hide()` | Close the modal |
| `setStage(indexOrId)` | Jump to stage by index or string id |
| `nextStage()` | Advance to next non-skipped stage |
| `prevStage()` | Go back to previous stage |
| `currentStageIndex` | Ref to current stage index |

## Non-Progress Stages (Edit Sub-Flows)

For stages that shouldn't appear in the progress bar (e.g. editing a specific field from a summary page):

```ts
export const editLoadersStageConfig: StageConfigInput<MyContext> = {
	id: 'edit-loaders',
	nonProgressStage: true,
	stageContent: markRaw(EditLoadersStage),
	title: 'Edit loaders',
	leftButtonConfig: (ctx) => ({
		label: 'Back',
		onClick: () => ctx.modal.value?.setStage('summary'),
	}),
	rightButtonConfig: (ctx) => ({
		...ctx.saveButtonConfig(),
		label: 'Save',
	}),
}
```

Navigate to it with `modal.value?.setStage('edit-loaders')` — it won't affect the progress indicator.

## Reference Implementation

The version creation/edit modal is the most complete example:

| File | Purpose |
|------|---------|
| `apps/frontend/src/providers/version/manage-version-modal.ts` | Context creation + business logic |
| `apps/frontend/src/providers/version/stages/index.ts` | Stage config barrel export |
| `apps/frontend/src/providers/version/stages/*-stage.ts` | Individual stage configs |

The context includes computed properties for conditional UI, watchers for auto-fetching dependencies, loading states for granular button disabling, and both "create" and "edit" flows sharing the same stages with different button configs.
