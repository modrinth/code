- [Standard modals](#standard-modals)
	- [Basic use](#basic-use)
	- [Props](#props)
	- [Slots](#slots)
		- [Default slot](#default-slot)
		- [`title` slot](#title-slot)
		- [`actions` slot](#actions-slot)
	- [Scrollable content](#scrollable-content)
	- [Merged header](#merged-header)
	- [Modal stack](#modal-stack)
	- [Exposed methods](#exposed-methods)
- [Multistage modals](#multistage-modals)
	- [Architecture](#architecture)
	- [Create a multistage modal](#create-a-multistage-modal)
		- [1. Define the context](#1-define-the-context)
		- [2. Define stage configurations](#2-define-stage-configurations)
		- [3. Create stage components](#3-create-stage-components)
		- [4. Create the wrapper component](#4-create-the-wrapper-component)
	- [Modal API](#modal-api)
	- [Non-progress stages](#non-progress-stages)
	- [Reference implementation](#reference-implementation)

# Standard Modals

Use `NewModal` (`packages/ui/src/components/modal/NewModal.vue`) for all standard modals.

- Set the modal width with the `width` or `maxWidth` prop.
- For a responsive width, use `min(base-size, calc(95vw - 10rem))`.
- Do not use `ModalWrapper`. The `injectModalBehavior` DI utility supplies modal behavior.

## Basic Use

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { NewModal } from '@modrinth/ui'

const modal = ref<InstanceType<typeof NewModal> | null>(null)
</script>

<template>
	<button @click="modal?.show($event)">Open</button>

	<NewModal ref="modal" header="My Modal">
		<p>Modal content.</p>
	</NewModal>
</template>
```

Call `show(event?)` to open the modal. A `MouseEvent` starts the animation at the click position.

Call `hide()` to close the modal from code.

## Props

| Prop                  | Type                                      | Default       | Description                                                        |
| --------------------- | ----------------------------------------- | ------------- | ------------------------------------------------------------------ |
| `header`              | `string`                                  | None          | Sets the title in the header bar.                                  |
| `hideHeader`          | `boolean`                                 | `false`       | Hides the title and close button.                                  |
| `mergeHeader`         | `boolean`                                 | `false`       | Replaces the header bar with a floating close button.              |
| `closable`            | `boolean`                                 | `true`        | Enables the close button, Escape key, and overlay click.           |
| `disableClose`        | `boolean`                                 | `false`       | Disables all close actions and shows a disabled close button.      |
| `closeOnEsc`          | `boolean`                                 | `true`        | Enables the Escape key as a close action.                          |
| `closeOnClickOutside` | `boolean`                                 | `true`        | Enables an overlay click as a close action.                        |
| `scrollable`          | `boolean`                                 | `false`       | Enables scroll tracking and edge-fade indicators.                  |
| `maxContentHeight`    | `string`                                  | `'70vh'`      | Sets the maximum scrollable-content height.                        |
| `noPadding`           | `boolean`                                 | `false`       | Removes content padding for edge-to-edge layouts.                  |
| `maxWidth`            | `string`                                  | `'60rem'`     | Sets the maximum modal width.                                      |
| `width`               | `string`                                  | `fit-content` | Sets the modal-body width.                                         |
| `noblur`              | `boolean`                                 | None          | Disables the backdrop blur. The DI behavior supplies the default.  |
| `fade`                | `'standard' \| 'warning' \| 'danger'`     | `'standard'`  | Sets the overlay color variant.                                    |
| `danger`              | `boolean`                                 | `false`       | Deprecated. Use `fade="danger"`.                                 |
| `onShow`              | `() => void`                              | None          | Runs when the modal opens.                                         |
| `onHide`              | `() => void`                              | None          | Runs when the modal closes.                                        |

`maxContentHeight` has an effect only when `scrollable` is true.

## Slots

### Default Slot

The default slot contains the main content. `NewModal` puts it in a padded container that can scroll.

```vue
<NewModal ref="modal" header="Confirm">
	<p>Are you sure that you want to continue?</p>
</NewModal>
```

### `title` Slot

The `title` slot replaces the default header text. Use it for custom header markup, such as an icon or badge.

```vue
<NewModal ref="modal">
	<template #title>
		<AlertIcon />
		<span class="text-2xl font-semibold text-contrast">Custom Title</span>
	</template>
	<p>Content.</p>
</NewModal>
```

### `actions` Slot

The `actions` slot makes an action bar below the content. The bar uses `p-4 pt-0` padding.

Use this slot for confirmation and cancellation buttons:

```vue
<NewModal ref="modal" header="Delete Item" fade="danger">
	<p>You cannot reverse this action.</p>
	<template #actions>
		<Button type="colored" color="red" @click="handleDelete">Delete</Button>
		<Button @click="modal?.hide()">Cancel</Button>
	</template>
</NewModal>
```

## Scrollable Content

Set `scrollable` to enable scroll tracking. Fade gradients appear at the top and bottom when more content exists.

```vue
<NewModal ref="modal" header="Long Content" scrollable max-content-height="60vh">
	<!-- Long content can overflow. -->
</NewModal>
```

Call the exposed `checkScrollState` method after a dynamic content change. The method recalculates the fade-indicator state.

When `scrollable` is false, the content uses `overflow-y: auto` without fade indicators. False is the default value.

## Merged Header

When `mergeHeader` is true, the header bar is hidden. A floating close button appears in the top-right corner.

The content gets more top padding. This padding prevents overlap with the button.

Use this mode for a hero image or full-width content at the top:

```vue
<NewModal ref="modal" merge-header no-padding>
	<img src="..." class="w-full" />
	<div class="p-6">
		<p>Content below the image.</p>
	</div>
</NewModal>
```

## Modal Stack

`NewModal` uses `useModalStack`. Multiple modals can be open at the same time.

Only the top modal responds to the Escape key. The first open modal locks document-body scrolling.

The last modal restores document-body scrolling when it closes.

## Exposed Methods

| Method               | Description                                                   |
| -------------------- | ------------------------------------------------------------- |
| `show(event?)`       | Opens the modal. Pass a `MouseEvent` for the origin animation. |
| `hide()`             | Closes the modal.                                             |
| `checkScrollState()` | Recalculates fade indicators when `scrollable` is true.       |

# Multistage Modals

`MultiStageModal` (`packages/ui/src/components/base/MultiStageModal.vue`) supplies progress, conditional stages, and button configurations for each stage.

## Architecture

A multistage modal has three parts:

1. The context contains all state, application logic, and stage configurations.
2. Stage configurations define the title, component, buttons, and skip conditions for each stage.
3. Stage components inject the context and render inside the modal.

## Create a Multistage Modal

### 1. Define the Context

Make a DI provider that contains the modal state. Include the modal reference and stage configurations.

```ts
// providers/my-feature/my-modal.ts
import type { ShallowRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'
import type { MultiStageModal, StageConfigInput } from '@modrinth/ui'
import { createContext } from '@modrinth/ui'

export interface MyModalContext {
	// State.
	formData: Ref<MyFormData>
	isSubmitting: Ref<boolean>

	// Modal control.
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>
	stageConfigs: StageConfigInput<MyModalContext>[]

	// Application logic.
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

### 2. Define Stage Configurations

Each stage is a `StageConfigInput<T>`, where `T` is the context type.

Most fields accept a static value or a function that receives the context. The function type is `MaybeCtxFn<T, R>`.

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

	// Set behavior from the context.
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

Stage configuration fields:

| Field                   | Type                                       | Purpose                                           |
| ----------------------- | ------------------------------------------ | ------------------------------------------------- |
| `id`                    | `string`                                   | Supplies the unique stage identifier.             |
| `stageContent`          | `Component`                                | Supplies the Vue component. Use `markRaw()`.      |
| `title`                 | `MaybeCtxFn<T, string>`                    | Supplies the breadcrumb title.                    |
| `skip`                  | `MaybeCtxFn<T, boolean>`                   | Skips the stage when the value is true.           |
| `nonProgressStage`      | `MaybeCtxFn<T, boolean>`                   | Removes the stage from the progress bar.          |
| `hideStageInBreadcrumb` | `MaybeCtxFn<T, boolean>`                   | Removes the stage from breadcrumb navigation.     |
| `cannotNavigateForward` | `MaybeCtxFn<T, boolean>`                   | Prevents forward navigation.                      |
| `disableClose`          | `MaybeCtxFn<T, boolean>`                   | Disables modal close actions.                     |
| `leftButtonConfig`      | `MaybeCtxFn<T, StageButtonConfig \| null>` | Configures the left action button.                |
| `rightButtonConfig`     | `MaybeCtxFn<T, StageButtonConfig \| null>` | Configures the right action button.               |
| `maxWidth`              | `MaybeCtxFn<T, string>`                    | Sets the stage width. The default is `560px`.     |

Button configuration fields:

| Field          | Purpose                                 |
| -------------- | --------------------------------------- |
| `label`        | Supplies the button text.               |
| `icon`         | Supplies the icon component.            |
| `iconPosition` | Uses `'before'` or `'after'`.            |
| `color`        | Supplies the `Button` color prop.       |
| `disabled`     | Disables the button when true.          |
| `onClick`      | Supplies the click handler.             |

### 3. Create Stage Components

Inject the context into each stage component. Then, render the applicable UI:

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

### 4. Create the Wrapper Component

Provide the context from the wrapper. Then, render `MultiStageModal`:

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

`MultiStageModal` exposes these methods and properties through its reference:

| Method or property     | Description                                  |
| ---------------------- | -------------------------------------------- |
| `show()`               | Opens the modal.                             |
| `hide()`               | Closes the modal.                            |
| `setStage(indexOrId)`  | Goes to a stage by index or string ID.       |
| `nextStage()`          | Goes to the next applicable stage.           |
| `prevStage()`          | Goes to the previous stage.                  |
| `currentStageIndex`    | Contains the current stage index as a `Ref`. |

## Non-Progress Stages

Use a non-progress stage for an edit flow that must not appear in the progress bar:

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

Call `modal.value?.setStage('edit-loaders')` to open the stage. This stage does not change the progress indicator.

## Reference Implementation

The version create-and-edit modal is the most complete example:

| File                                                          | Purpose                                |
| ------------------------------------------------------------- | -------------------------------------- |
| `apps/frontend/src/providers/version/manage-version-modal.ts` | Contains context and application logic. |
| `apps/frontend/src/providers/version/stages/index.ts`         | Exports all stage configurations.       |
| `apps/frontend/src/providers/version/stages/*-stage.ts`       | Contains each stage configuration.      |

The context has computed properties for conditional UI. It also has dependency watchers and granular button loading states.

The create and edit flows use the same stages with different button configurations.
