- [Regular Modals](#regular-modals)
  - [Basic Usage](#basic-usage)
  - [Props](#props)
  - [Slots](#slots)
    - [Default slot](#default-slot)
    - [`title` slot](#title-slot)
    - [`actions` slot](#actions-slot)
  - [Scrollable Content](#scrollable-content)
  - [Merged Header Mode](#merged-header-mode)
  - [Modal Stacking](#modal-stacking)
  - [Exposed Methods](#exposed-methods)
- [Multistage Modals](#multistage-modals)
  - [Architecture](#architecture)
  - [Building a Multistage Modal](#building-a-multistage-modal)
    - [1. Define the context](#1-define-the-context)
    - [2. Define stage configs](#2-define-stage-configs)
    - [3. Create stage components](#3-create-stage-components)
    - [4. Create the wrapper component](#4-create-the-wrapper-component)
  - [Modal API](#modal-api)
  - [Non-Progress Stages (Edit Sub-Flows)](#non-progress-stages-edit-sub-flows)
  - [Reference Implementation](#reference-implementation)

# Regular Modals

Use the `NewModal` component (`packages/ui/src/components/modal/NewModal.vue`) for all standard modals.

- Set the modal’s width via the `width` or `maxWidth` props. For responsive sizing, use `min(base-size, calc(95vw - 10rem))`.
- `ModalWrapper` is deprecated — modal behavior is automatically handled via the `injectModalBehavior` DI utility.

## Basic Usage

```vue
<script setup lang="ts">
import { ref } from ‘vue’
import { NewModal } from ‘@modrinth/ui’

const modal = ref<InstanceType<typeof NewModal> | null>(null)
</script>

<template>
	<button @click="modal?.show($event)">Open</button>

	<NewModal ref="modal" header="My Modal">
		<p>Modal content here.</p>
	</NewModal>
</template>
```

Call `show(event?)` to open the modal. Passing the `MouseEvent` triggers an animation originating from the click position. Call `hide()` to close it programmatically.

## Props

| Prop                  | Type                                  | Default       | Description                                                                                      |
| --------------------- | ------------------------------------- | ------------- | ------------------------------------------------------------------------------------------------ |
| `header`              | `string`                              | —             | Title text displayed in the header bar                                                           |
| `hideHeader`          | `boolean`                             | `false`       | Hides the entire header (title + close button)                                                   |
| `mergeHeader`         | `boolean`                             | `false`       | Removes the header bar; renders a floating close button over the content                         |
| `closable`            | `boolean`                             | `true`        | Shows the close button and enables ESC / click-outside dismissal                                 |
| `disableClose`        | `boolean`                             | `false`       | Disables all close actions (close button, ESC, click-outside). The close button appears disabled |
| `closeOnEsc`          | `boolean`                             | `true`        | Allow closing with the Escape key                                                                |
| `closeOnClickOutside` | `boolean`                             | `true`        | Allow closing by clicking the overlay                                                            |
| `scrollable`          | `boolean`                             | `false`       | Enables scroll tracking with top/bottom fade indicators                                          |
| `maxContentHeight`    | `string`                              | `’70vh’`      | Max height of the scrollable content area (only applies when `scrollable`)                       |
| `noPadding`           | `boolean`                             | `false`       | Removes padding from the content area for edge-to-edge layouts                                   |
| `maxWidth`            | `string`                              | `’60rem’`     | Maximum width of the modal                                                                       |
| `width`               | `string`                              | `fit-content` | Width of the modal body                                                                          |
| `noblur`              | `boolean`                             | —             | Disables backdrop blur. Defaults to the value from `injectModalBehavior`                         |
| `fade`                | `’standard’ \| ‘warning’ \| ‘danger’` | `’standard’`  | Overlay color variant                                                                            |
| `danger`              | `boolean`                             | `false`       | **Deprecated** — use `fade="danger"` instead                                                     |
| `onShow`              | `() => void`                          | —             | Called when the modal opens                                                                      |
| `onHide`              | `() => void`                          | —             | Called when the modal closes                                                                     |

## Slots

### Default slot

The main content area. Rendered inside a padded, optionally scrollable container.

```vue
<NewModal ref="modal" header="Confirm">
	<p>Are you sure you want to proceed?</p>
</NewModal>
```

### `title` slot

Replaces the default header text. Use this when you need custom markup in the header (e.g. an icon next to the title or a badge).

```vue
<NewModal ref="modal">
	<template #title>
		<AlertIcon />
		<span class="text-2xl font-semibold text-contrast">Custom Title</span>
	</template>
	<p>Content here.</p>
</NewModal>
```

### `actions` slot

Renders a bottom action bar below the content area (with `p-4 pt-0` padding). Use this for confirm/cancel buttons.

```vue
<NewModal ref="modal" header="Delete Item" fade="danger">
	<p>This action cannot be undone.</p>
	<template #actions>
		<ButtonStyled color="danger">
			<button @click="handleDelete">Delete</button>
		</ButtonStyled>
		<ButtonStyled>
			<button @click="modal?.hide()">Cancel</button>
		</ButtonStyled>
	</template>
</NewModal>
```

## Scrollable Content

Set `scrollable` to enable scroll tracking. The modal renders animated fade gradients at the top and bottom edges when content is scrolled, giving users a visual cue that more content exists.

```vue
<NewModal ref="modal" header="Long Content" scrollable max-content-height="60vh">
	<!-- Long content that may overflow -->
</NewModal>
```

The `checkScrollState` method is exposed via ref — call it after dynamically changing content to re-evaluate whether fade indicators should appear.

When `scrollable` is `false` (the default), content uses `overflow-y: auto` without fade indicators.

## Merged Header Mode

When `mergeHeader` is set, the header bar is hidden and a floating close button is rendered in the top-right corner of the modal. Content receives extra top padding to avoid overlapping the button. This is useful for modals with hero images or full-bleed content at the top.

```vue
<NewModal ref="modal" merge-header no-padding>
	<img src="..." class="w-full" />
	<div class="p-6">
		<p>Content below the image.</p>
	</div>
</NewModal>
```

## Modal Stacking

`NewModal` integrates with a modal stack (`useModalStack`). Multiple modals can be open simultaneously — only the topmost modal responds to the Escape key. The document body scroll is locked when any modal is open and restored when the last modal closes.

## Exposed Methods

| Method               | Description                                             |
| -------------------- | ------------------------------------------------------- |
| `show(event?)`       | Opens the modal. Pass `MouseEvent` for origin animation |
| `hide()`             | Closes the modal                                        |
| `checkScrollState()` | Re-evaluates scroll fade indicators (when `scrollable`) |

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

| Field                   | Type                                       | Purpose                                          |
| ----------------------- | ------------------------------------------ | ------------------------------------------------ |
| `id`                    | `string`                                   | Unique stage identifier (used with `setStage()`) |
| `stageContent`          | `Component`                                | Vue component to render (wrap with `markRaw()`)  |
| `title`                 | `MaybeCtxFn<T, string>`                    | Stage title in breadcrumbs                       |
| `skip`                  | `MaybeCtxFn<T, boolean>`                   | Skip this stage conditionally                    |
| `nonProgressStage`      | `MaybeCtxFn<T, boolean>`                   | Exclude from progress bar (for edit sub-flows)   |
| `hideStageInBreadcrumb` | `MaybeCtxFn<T, boolean>`                   | Hide from breadcrumb nav                         |
| `cannotNavigateForward` | `MaybeCtxFn<T, boolean>`                   | Block forward navigation (validation)            |
| `disableClose`          | `MaybeCtxFn<T, boolean>`                   | Disable closing the modal                        |
| `leftButtonConfig`      | `MaybeCtxFn<T, StageButtonConfig \| null>` | Left action button                               |
| `rightButtonConfig`     | `MaybeCtxFn<T, StageButtonConfig \| null>` | Right action button                              |
| `maxWidth`              | `MaybeCtxFn<T, string>`                    | Per-stage max width (default `560px`)            |

**Button config fields:**

| Field          | Purpose                 |
| -------------- | ----------------------- |
| `label`        | Button text             |
| `icon`         | Icon component          |
| `iconPosition` | `'before'` or `'after'` |
| `color`        | ButtonStyled color prop |
| `disabled`     | Disable the button      |
| `onClick`      | Click handler           |

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

| Method/Property       | Description                         |
| --------------------- | ----------------------------------- |
| `show()`              | Open the modal                      |
| `hide()`              | Close the modal                     |
| `setStage(indexOrId)` | Jump to stage by index or string id |
| `nextStage()`         | Advance to next non-skipped stage   |
| `prevStage()`         | Go back to previous stage           |
| `currentStageIndex`   | Ref to current stage index          |

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

| File                                                          | Purpose                           |
| ------------------------------------------------------------- | --------------------------------- |
| `apps/frontend/src/providers/version/manage-version-modal.ts` | Context creation + business logic |
| `apps/frontend/src/providers/version/stages/index.ts`         | Stage config barrel export        |
| `apps/frontend/src/providers/version/stages/*-stage.ts`       | Individual stage configs          |

The context includes computed properties for conditional UI, watchers for auto-fetching dependencies, loading states for granular button disabling, and both "create" and "edit" flows sharing the same stages with different button configs.
