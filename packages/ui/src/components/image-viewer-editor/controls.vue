<script setup lang="ts">
import {
	RedoIcon,
	SaveIcon,
	TrashIcon,
	UndoIcon,
	XIcon,
	ZoomInIcon,
	ZoomOutIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import Button from '#ui/components/base/buttons/Button.vue'
import IconButton from '#ui/components/base/buttons/IconButton.vue'
import SplitButton from '#ui/components/base/buttons/SplitButton.vue'
import type { OverflowMenuOption } from '#ui/components/base/buttons/types'
import Chips from '#ui/components/base/Chips.vue'
import { useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { imageViewerEditorMessages as messages } from './image-viewer-editor-messages'
import type { ScreenshotCensorMode, ScreenshotEraserMode } from './image-viewer-editor-types'
import type { useImageEditor } from './use-image-editor'

const props = defineProps<{
	editor: ReturnType<typeof useImageEditor>
	busy: boolean
}>()

const emit = defineEmits<{
	cancel: []
	save: [mode: 'create_copy' | 'replace_edit']
}>()

const { formatMessage } = useVIntl()
const {
	color,
	strokeWidth,
	fontSize,
	censorMode,
	eraserMode,
	zoom,
	isFit,
	canUndo,
	canRedo,
	canDelete,
	canZoomOut,
	canZoomIn,
	hasColorProperty,
	propertyValueKind,
	showCensorMode,
	showEraserMode,
	showCropControls,
	cropWidth,
	cropHeight,
	canResetCrop,
	updateColor,
	updateStrokeWidth,
	updateFontSize,
	beginPropertyEdit,
	commitPropertyEdit,
	deleteSelection,
	resetCrop,
	undo,
	redo,
	setZoom,
	setFit,
} = props.editor

const propertyValue = computed(() =>
	propertyValueKind.value === 'size' ? fontSize.value : strokeWidth.value,
)
const propertyMin = computed(() => (propertyValueKind.value === 'size' ? 12 : 1))
const propertyMax = computed(() => (propertyValueKind.value === 'size' ? 120 : 40))
const propertyProgress = computed(
	() => ((propertyValue.value - propertyMin.value) / (propertyMax.value - propertyMin.value)) * 100,
)
const hasPropertyControls = computed(
	() =>
		showCropControls.value ||
		showEraserMode.value ||
		showCensorMode.value ||
		hasColorProperty.value ||
		Boolean(propertyValueKind.value),
)
const censorModes: ScreenshotCensorMode[] = ['blur', 'solid']
const eraserModes: ScreenshotEraserMode[] = ['element', 'area']
const saveOptions = computed<OverflowMenuOption[]>(() => [
	{
		id: 'overwrite',
		label: formatMessage(messages.overwrite),
		icon: SaveIcon,
		action: () => emit('save', 'replace_edit'),
	},
])

function updatePropertyValue(nextValue: number) {
	if (!Number.isFinite(nextValue)) return
	const normalizedValue = Math.min(
		propertyMax.value,
		Math.max(propertyMin.value, Math.round(nextValue)),
	)
	if (propertyValueKind.value === 'size') updateFontSize(normalizedValue)
	else updateStrokeWidth(normalizedValue)
}

function handlePropertyInput(event: Event) {
	updatePropertyValue(Number((event.target as HTMLInputElement).value))
}

function handleColorInput(event: Event) {
	updateColor((event.target as HTMLInputElement).value)
}

function formatCensorMode(value: ScreenshotCensorMode) {
	return formatMessage(value === 'blur' ? messages.blur : messages.solid)
}

function formatEraserMode(value: ScreenshotEraserMode) {
	return formatMessage(value === 'element' ? messages.element : messages.area)
}
</script>

<template>
	<div
		class="absolute bottom-6 left-1/2 z-10 flex w-max max-w-[calc(100%_-_3rem)] -translate-x-1/2 items-center gap-2 rounded-[20px] border border-solid border-white/10 bg-surface-3 p-2 shadow-[0_1rem_3rem_rgb(0_0_0_/_32%)] max-[900px]:flex-wrap max-[900px]:justify-center"
		@click.stop
	>
		<div v-if="hasPropertyControls" class="flex min-w-0 items-center gap-2">
			<div v-if="showCropControls" class="flex items-center gap-2">
				<span class="shrink-0 text-base font-semibold leading-5 tabular-nums text-white/60">
					{{ formatMessage(messages.cropDimensions, { width: cropWidth, height: cropHeight }) }}
				</span>
				<Button type="quiet" :disabled="!canResetCrop" @click="resetCrop">
					{{ formatMessage(messages.resetCrop) }}
				</Button>
			</div>
			<Chips
				v-if="showEraserMode"
				v-model="eraserMode"
				:items="eraserModes"
				:format-label="formatEraserMode"
				:aria-label="formatMessage(messages.eraserMode)"
				size="small"
				hide-checkmark-icon
			/>
			<Chips
				v-if="showCensorMode"
				v-model="censorMode"
				:items="censorModes"
				:format-label="formatCensorMode"
				:aria-label="formatMessage(messages.censorMode)"
				size="small"
				hide-checkmark-icon
			/>
			<label
				v-if="hasColorProperty"
				v-tooltip="formatMessage(messages.colour)"
				class="relative flex size-9 shrink-0 cursor-pointer items-center justify-center rounded-xl transition-[filter,box-shadow] hover:brightness-125 focus-within:ring-4 focus-within:ring-brand-shadow"
			>
				<span
					class="size-full rounded-xl border border-solid border-surface-5"
					:style="{ backgroundColor: color }"
				/>
				<input
					:value="color"
					type="color"
					:aria-label="formatMessage(messages.colour)"
					class="absolute inset-0 cursor-pointer opacity-0"
					@focus="beginPropertyEdit"
					@pointerdown="beginPropertyEdit"
					@input="handleColorInput"
					@change="commitPropertyEdit"
					@blur="commitPropertyEdit"
				/>
			</label>
			<div
				v-if="propertyValueKind"
				class="flex w-52 min-w-0 items-center gap-2"
				@focusin="beginPropertyEdit"
				@pointerdown.capture="beginPropertyEdit"
				@change.capture="commitPropertyEdit"
				@focusout="commitPropertyEdit"
			>
				<span class="shrink-0 text-base font-semibold leading-5 text-white/60">
					{{ formatMessage(propertyValueKind === 'size' ? messages.size : messages.width) }}
				</span>
				<input
					type="range"
					:value="propertyValue"
					:min="propertyMin"
					:max="propertyMax"
					step="1"
					:aria-label="formatMessage(propertyValueKind === 'size' ? messages.size : messages.width)"
					class="editor-property-range min-w-0 flex-1"
					:style="{ '--property-progress': `${propertyProgress}%` }"
					@input="handlePropertyInput"
				/>
				<input
					type="number"
					:value="propertyValue"
					:min="propertyMin"
					:max="propertyMax"
					step="1"
					:aria-label="formatMessage(propertyValueKind === 'size' ? messages.size : messages.width)"
					class="editor-property-value h-9 w-12 shrink-0 rounded-xl border border-solid border-surface-5 bg-surface-4 px-1 text-center text-base font-semibold leading-5 tabular-nums text-white outline-none focus:ring-4 focus:ring-brand-shadow"
					@input="handlePropertyInput"
				/>
			</div>
		</div>

		<div v-if="hasPropertyControls" class="h-6 w-px bg-white/10" />
		<div class="flex items-center gap-2">
			<IconButton
				v-if="canDelete"
				v-tooltip="formatMessage(messages.deleteSelection)"
				:label="formatMessage(messages.deleteSelection)"
				type="quiet"
				@click="deleteSelection"
			>
				<TrashIcon />
			</IconButton>
			<IconButton
				v-tooltip="formatMessage(messages.undo)"
				:label="formatMessage(messages.undo)"
				type="quiet"
				:disabled="!canUndo"
				@click="undo"
			>
				<UndoIcon />
			</IconButton>
			<IconButton
				v-tooltip="formatMessage(messages.redo)"
				:label="formatMessage(messages.redo)"
				type="quiet"
				:disabled="!canRedo"
				@click="redo"
			>
				<RedoIcon />
			</IconButton>
			<IconButton
				v-tooltip="formatMessage(messages.zoomOut)"
				:label="formatMessage(messages.zoomOut)"
				type="quiet"
				:disabled="!canZoomOut"
				@click="setZoom(zoom - 0.1)"
			>
				<ZoomOutIcon />
			</IconButton>
			<Button
				v-tooltip="formatMessage(messages.fitToWorkspace)"
				type="quiet"
				class="w-16 px-2 tabular-nums text-white/60"
				@click="setFit"
			>
				{{ isFit ? formatMessage(messages.fit) : `${Math.round(zoom * 100)}%` }}
			</Button>
			<IconButton
				v-tooltip="formatMessage(messages.zoomIn)"
				:label="formatMessage(messages.zoomIn)"
				type="quiet"
				:disabled="!canZoomIn"
				@click="setZoom(zoom + 0.1)"
			>
				<ZoomInIcon />
			</IconButton>
		</div>

		<div class="h-6 w-px bg-white/10" />
		<Button type="quiet" :disabled="busy" @click="emit('cancel')">
			<XIcon aria-hidden="true" />
			{{ formatMessage(commonMessages.cancelButton) }}
		</Button>
		<SplitButton
			type="colored"
			color="green"
			:disabled="busy"
			:options="saveOptions"
			:menu-label="formatMessage(messages.moreSaveOptions)"
			:group-label="formatMessage(messages.saveImage)"
			@click="emit('save', 'create_copy')"
		>
			<SaveIcon aria-hidden="true" />
			{{ formatMessage(messages.saveAsCopy) }}
		</SplitButton>
	</div>
</template>

<style scoped>
.editor-property-range {
	height: 1rem;
	padding: 0;
	margin: 0;
	cursor: pointer;
	background: transparent;
	outline: none;
	appearance: none;

	&::-webkit-slider-runnable-track {
		height: 0.25rem;
		border-radius: 999px;
		background: linear-gradient(
			to right,
			var(--color-brand) 0%,
			var(--color-brand) var(--property-progress),
			var(--surface-5) var(--property-progress),
			var(--surface-5) 100%
		);
	}

	&::-webkit-slider-thumb {
		width: 0.875rem;
		height: 0.875rem;
		margin-top: -0.3125rem;
		border: 2px solid var(--surface-1);
		border-radius: 999px;
		background: var(--color-brand);
		box-shadow: 0 1px 3px rgb(0 0 0 / 35%);
		appearance: none;
	}

	&:focus-visible::-webkit-slider-thumb {
		box-shadow: 0 0 0 0.25rem var(--color-brand-shadow);
	}

	&::-moz-range-track {
		height: 0.25rem;
		border-radius: 999px;
		background: var(--surface-5);
	}

	&::-moz-range-progress {
		height: 0.25rem;
		border-radius: 999px;
		background: var(--color-brand);
	}

	&::-moz-range-thumb {
		width: 0.75rem;
		height: 0.75rem;
		border: 2px solid var(--surface-1);
		border-radius: 999px;
		background: var(--color-brand);
		box-shadow: 0 1px 3px rgb(0 0 0 / 35%);
	}
}

.editor-property-value {
	appearance: textfield;

	&::-webkit-inner-spin-button,
	&::-webkit-outer-spin-button {
		margin: 0;
		appearance: none;
	}
}
</style>
