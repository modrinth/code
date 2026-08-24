<script setup lang="ts">
import {
	CircleIcon,
	EyeOffIcon,
	HighlighterIcon,
	MousePointer2Icon,
	MoveUpRightIcon,
	PencilIcon,
	RedoIcon,
	SaveIcon,
	SquareIcon,
	TrashIcon,
	TypeIcon,
	UndoIcon,
	ZoomInIcon,
	ZoomOutIcon,
} from '@modrinth/assets'
import {
	Button,
	Chips,
	defineMessages,
	IconButton,
	injectNotificationManager,
	NewModal,
	type OverflowMenuOption,
	SplitButton,
	useVIntl,
} from '@modrinth/ui'
import { computed, nextTick, onBeforeUnmount, ref } from 'vue'

import type { InstanceScreenshot } from '@/helpers/instance'

import type { ScreenshotCensorMode, ScreenshotEditorTool } from './editor-types'
import { useScreenshotEditor } from './use-screenshot-editor'

const props = defineProps<{
	saving: boolean
}>()

const emit = defineEmits<{
	save: [
		payload: {
			screenshot: InstanceScreenshot
			pngBytes: Uint8Array
			mode: 'create_copy' | 'replace_edit'
		},
	]
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const canvasElement = ref<HTMLCanvasElement>()
const viewport = ref<HTMLElement>()
const screenshot = ref<InstanceScreenshot>()
const exporting = ref(false)
const closingAfterSave = ref(false)
const spacePressed = ref(false)
const panning = ref<{ x: number; y: number; scrollLeft: number; scrollTop: number }>()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const {
	loading,
	tool,
	color,
	strokeWidth,
	fontSize,
	censorMode,
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
	initialize,
	dispose,
	setTool,
	updateColor,
	updateStrokeWidth,
	updateFontSize,
	beginPropertyEdit,
	commitPropertyEdit,
	deleteSelection,
	undo,
	redo,
	fitToViewport,
	setZoom,
	setFit,
	exportPng,
	handleKeyboardShortcut,
	isTextEditing,
	resetHistory,
} = useScreenshotEditor()

const messages = defineMessages({
	editScreenshot: { id: 'app.screenshots.editor.title', defaultMessage: 'Edit screenshot' },
	saveCopy: {
		id: 'app.screenshots.editor.save-copy',
		defaultMessage: 'Save copy',
	},
	saveChanges: { id: 'app.screenshots.editor.save-changes', defaultMessage: 'Save changes' },
	saveAsCopy: {
		id: 'app.screenshots.editor.save-as-new',
		defaultMessage: 'Save as copy',
	},
	moreSaveOptions: {
		id: 'app.screenshots.editor.more-save-options',
		defaultMessage: 'More save options',
	},
	saveScreenshot: {
		id: 'app.screenshots.editor.save-screenshot',
		defaultMessage: 'Save screenshot',
	},
	select: { id: 'app.screenshots.editor.tool.select', defaultMessage: 'Select' },
	pen: { id: 'app.screenshots.editor.tool.pen', defaultMessage: 'Pen' },
	highlight: { id: 'app.screenshots.editor.tool.highlight', defaultMessage: 'Highlight' },
	text: { id: 'app.screenshots.editor.tool.text', defaultMessage: 'Text' },
	arrow: { id: 'app.screenshots.editor.tool.arrow', defaultMessage: 'Arrow' },
	shape: { id: 'app.screenshots.editor.tool.shape', defaultMessage: 'Shape' },
	rectangle: { id: 'app.screenshots.editor.tool.rectangle', defaultMessage: 'Rectangle' },
	ellipse: { id: 'app.screenshots.editor.tool.ellipse', defaultMessage: 'Ellipse' },
	censor: { id: 'app.screenshots.editor.tool.censor', defaultMessage: 'Censor' },
	blur: { id: 'app.screenshots.editor.censor.blur', defaultMessage: 'Blur' },
	solid: { id: 'app.screenshots.editor.censor.solid', defaultMessage: 'Solid' },
	censorMode: { id: 'app.screenshots.editor.censor.mode', defaultMessage: 'Censor type' },
	colour: { id: 'app.screenshots.editor.colour', defaultMessage: 'Colour' },
	width: { id: 'app.screenshots.editor.width', defaultMessage: 'Width' },
	size: { id: 'app.screenshots.editor.size', defaultMessage: 'Size' },
	undo: { id: 'app.screenshots.editor.undo', defaultMessage: 'Undo' },
	redo: { id: 'app.screenshots.editor.redo', defaultMessage: 'Redo' },
	deleteSelection: {
		id: 'app.screenshots.editor.delete-selection',
		defaultMessage: 'Delete selected annotation',
	},
	zoomIn: { id: 'app.screenshots.editor.zoom-in', defaultMessage: 'Zoom in' },
	zoomOut: { id: 'app.screenshots.editor.zoom-out', defaultMessage: 'Zoom out' },
	fit: { id: 'app.screenshots.editor.zoom-fit', defaultMessage: 'Fit' },
	fitToWorkspace: {
		id: 'app.screenshots.editor.zoom-fit-workspace',
		defaultMessage: 'Fit to workspace',
	},
})

const busy = computed(() => props.saving || exporting.value || loading.value)
const isEditedScreenshot = computed(() => Boolean(screenshot.value?.original_screenshot_id))
const saveOptions = computed<OverflowMenuOption[]>(() => [
	{
		id: 'save-as-copy',
		label: formatMessage(messages.saveAsCopy),
		icon: SaveIcon,
		action: () => requestSave('create_copy'),
	},
])

type ScreenshotEditorToolOption = {
	id: ScreenshotEditorTool
	message: (typeof messages)[keyof typeof messages]
	icon: typeof MousePointer2Icon
	shortcut: string
}

const toolGroups: ScreenshotEditorToolOption[][] = [
	[{ id: 'select', message: messages.select, icon: MousePointer2Icon, shortcut: 'V' }],
	[
		{ id: 'pen', message: messages.pen, icon: PencilIcon, shortcut: 'P' },
		{ id: 'highlight', message: messages.highlight, icon: HighlighterIcon, shortcut: 'H' },
	],
	[
		{ id: 'arrow', message: messages.arrow, icon: MoveUpRightIcon, shortcut: 'A' },
		{ id: 'rectangle', message: messages.rectangle, icon: SquareIcon, shortcut: 'R' },
		{ id: 'ellipse', message: messages.ellipse, icon: CircleIcon, shortcut: 'O' },
	],
	[
		{ id: 'text', message: messages.text, icon: TypeIcon, shortcut: 'T' },
		{ id: 'censor', message: messages.censor, icon: EyeOffIcon, shortcut: 'C' },
	],
]

const propertyValue = computed(() =>
	propertyValueKind.value === 'size' ? fontSize.value : strokeWidth.value,
)
const propertyMin = computed(() => (propertyValueKind.value === 'size' ? 12 : 1))
const propertyMax = computed(() => (propertyValueKind.value === 'size' ? 120 : 40))
const propertyProgress = computed(
	() => ((propertyValue.value - propertyMin.value) / (propertyMax.value - propertyMin.value)) * 100,
)
const censorModes: ScreenshotCensorMode[] = ['blur', 'solid']

let resizeObserver: ResizeObserver | undefined

async function show(nextScreenshot: InstanceScreenshot) {
	screenshot.value = nextScreenshot
	closingAfterSave.value = false
	modal.value?.show()
	await nextTick()
	if (!canvasElement.value) return

	try {
		await initialize(canvasElement.value, nextScreenshot)
		observeViewport()
		listenForKeyboard()
	} catch (error) {
		handleError(error)
		modal.value?.hide()
	}
}

function hide() {
	modal.value?.hide()
}

async function markSavedAndHide() {
	closingAfterSave.value = true
	resetHistory()
	await nextTick()
	modal.value?.hide()
}

function cleanup() {
	resizeObserver?.disconnect()
	resizeObserver = undefined
	unlistenForKeyboard()
	dispose()
	screenshot.value = undefined
	closingAfterSave.value = false
	panning.value = undefined
}

function observeViewport() {
	resizeObserver?.disconnect()
	if (!viewport.value) return
	resizeObserver = new ResizeObserver(([entry]) => {
		if (entry) fitToViewport(entry.contentRect.width, entry.contentRect.height)
	})
	resizeObserver.observe(viewport.value)
	const styles = getComputedStyle(viewport.value)
	fitToViewport(
		viewport.value.clientWidth -
			Number.parseFloat(styles.paddingLeft) -
			Number.parseFloat(styles.paddingRight),
		viewport.value.clientHeight -
			Number.parseFloat(styles.paddingTop) -
			Number.parseFloat(styles.paddingBottom),
	)
}

function toolTooltip(editorTool: ScreenshotEditorToolOption) {
	return `${formatMessage(editorTool.message)} (${editorTool.shortcut})`
}

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

function formatCensorMode(mode: ScreenshotCensorMode) {
	return formatMessage(mode === 'blur' ? messages.blur : messages.solid)
}

function handleColorInput(event: Event) {
	updateColor((event.target as HTMLInputElement).value)
}

async function requestSave(mode: 'create_copy' | 'replace_edit') {
	if (busy.value || !screenshot.value) return
	exporting.value = true
	try {
		emit('save', {
			screenshot: screenshot.value,
			pngBytes: await exportPng(),
			mode,
		})
	} catch (error) {
		handleError(error)
	} finally {
		exporting.value = false
	}
}

function handleKeydown(event: KeyboardEvent) {
	if (handleKeyboardShortcut(event)) return
	if (event.code === 'Space' && !isTextEditing() && !isTypingTarget(event.target)) {
		spacePressed.value = true
	}
	if (event.key === 'Escape') {
		event.preventDefault()
		modal.value?.hide()
	}
}

function handleKeyup(event: KeyboardEvent) {
	if (event.code === 'Space') spacePressed.value = false
}

function listenForKeyboard() {
	document.addEventListener('keydown', handleKeydown)
	document.addEventListener('keyup', handleKeyup)
}

function unlistenForKeyboard() {
	document.removeEventListener('keydown', handleKeydown)
	document.removeEventListener('keyup', handleKeyup)
}

function startPan(event: PointerEvent) {
	if (!viewport.value || (event.button !== 1 && !(event.button === 0 && spacePressed.value))) return
	event.preventDefault()
	event.stopPropagation()
	panning.value = {
		x: event.clientX,
		y: event.clientY,
		scrollLeft: viewport.value.scrollLeft,
		scrollTop: viewport.value.scrollTop,
	}
	viewport.value.setPointerCapture(event.pointerId)
}

function movePan(event: PointerEvent) {
	if (!viewport.value || !panning.value) return
	viewport.value.scrollLeft = panning.value.scrollLeft - (event.clientX - panning.value.x)
	viewport.value.scrollTop = panning.value.scrollTop - (event.clientY - panning.value.y)
}

function stopPan() {
	panning.value = undefined
}

function handleWheel(event: WheelEvent) {
	if (!event.ctrlKey && !event.metaKey) return
	event.preventDefault()
	setZoom(zoom.value + (event.deltaY < 0 ? 0.1 : -0.1))
}

function isTypingTarget(target: EventTarget | null) {
	return (
		target instanceof HTMLInputElement ||
		target instanceof HTMLTextAreaElement ||
		target instanceof HTMLSelectElement ||
		(target instanceof HTMLElement && target.isContentEditable)
	)
}

onBeforeUnmount(cleanup)

defineExpose({ show, hide, markSavedAndHide })
</script>

<template>
	<NewModal
		ref="modal"
		no-padding
		:close-on-click-outside="false"
		:close-on-esc="false"
		:on-after-hide="cleanup"
		:disable-close="busy && !closingAfterSave"
		width="calc(100vw - 3rem)"
		max-width="calc(100vw - 3rem)"
		class="h-[calc(100dvh-7rem)]"
	>
		<template #title>
			<div class="flex min-w-0 flex-col">
				<span class="text-xl font-semibold text-contrast">
					{{ formatMessage(messages.editScreenshot) }}
				</span>
				<span class="truncate text-sm font-normal text-secondary">
					{{ screenshot?.file_name }}
				</span>
			</div>
		</template>
		<template #header-actions>
			<SplitButton
				v-if="isEditedScreenshot"
				type="colored"
				color="green"
				:disabled="busy"
				:options="saveOptions"
				:menu-label="formatMessage(messages.moreSaveOptions)"
				:group-label="formatMessage(messages.saveScreenshot)"
				@click="requestSave('replace_edit')"
			>
				{{ formatMessage(messages.saveChanges) }}
				<SaveIcon aria-hidden="true" />
			</SplitButton>
			<Button
				v-else
				type="colored"
				color="green"
				:disabled="busy"
				@click="requestSave('create_copy')"
			>
				{{ formatMessage(messages.saveCopy) }}
				<SaveIcon aria-hidden="true" />
			</Button>
		</template>

		<div class="flex h-[calc(100dvh-12.75rem)] min-h-0 flex-col">
			<div
				class="flex flex-wrap items-center gap-1 border-0 border-b border-solid border-surface-5 px-4 py-2"
			>
				<template v-for="(toolGroup, groupIndex) in toolGroups" :key="toolGroup[0]?.id">
					<div v-if="groupIndex > 0" class="mx-1 h-6 w-px bg-surface-5" />
					<IconButton
						v-for="editorTool in toolGroup"
						:key="editorTool.id"
						v-tooltip="toolTooltip(editorTool)"
						:label="formatMessage(editorTool.message)"
						:type="tool === editorTool.id ? 'colored' : 'quiet'"
						:color="tool === editorTool.id ? 'brand' : undefined"
						:aria-pressed="tool === editorTool.id"
						@click="setTool(editorTool.id)"
					>
						<component :is="editorTool.icon" />
					</IconButton>
				</template>

				<div class="mx-2 h-6 w-px bg-surface-5" />
				<div class="flex w-[19rem] shrink-0 items-center gap-2">
					<Chips
						v-if="showCensorMode"
						v-model="censorMode"
						:items="censorModes"
						:format-label="formatCensorMode"
						:aria-label="formatMessage(messages.censorMode)"
						size="small"
						hide-checkmark-icon
						class="shrink-0"
					/>
					<label
						v-if="hasColorProperty"
						v-tooltip="formatMessage(messages.colour)"
						class="relative flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-lg bg-surface-4 p-1.5 shadow-[inset_0_0_0_1px_var(--surface-5)] transition-[filter,box-shadow] hover:brightness-125 focus-within:ring-4 focus-within:ring-brand-shadow"
					>
						<span
							class="size-full rounded border border-solid border-black/20"
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
						class="flex min-w-0 flex-1 items-center gap-2"
						@focusin="beginPropertyEdit"
						@pointerdown.capture="beginPropertyEdit"
						@change.capture="commitPropertyEdit"
						@focusout="commitPropertyEdit"
					>
						<span class="shrink-0 text-xs font-medium text-secondary">
							{{ formatMessage(propertyValueKind === 'size' ? messages.size : messages.width) }}
						</span>
						<input
							type="range"
							:value="propertyValue"
							:min="propertyMin"
							:max="propertyMax"
							step="1"
							:aria-label="
								formatMessage(propertyValueKind === 'size' ? messages.size : messages.width)
							"
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
							:aria-label="
								formatMessage(propertyValueKind === 'size' ? messages.size : messages.width)
							"
							class="editor-property-value h-8 w-11 shrink-0 rounded-lg border-0 bg-surface-4 px-1 text-center text-sm font-semibold tabular-nums text-contrast outline-none shadow-[inset_0_0_0_1px_var(--surface-5)] focus:ring-4 focus:ring-brand-shadow"
							@input="handlePropertyInput"
						/>
					</div>
				</div>

				<div class="ml-auto flex items-center gap-1">
					<IconButton
						v-tooltip="formatMessage(messages.deleteSelection)"
						:label="formatMessage(messages.deleteSelection)"
						type="quiet"
						:disabled="!canDelete"
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
					<div class="mx-1 h-6 w-px bg-surface-5" />
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
						size="sm"
						class="w-14 px-1 text-sm tabular-nums text-secondary"
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
			</div>

			<div
				ref="viewport"
				class="editor-viewport relative min-h-0 flex-1 overflow-auto bg-surface-1 p-6"
				:class="{ 'is-panning': panning, 'is-pan-ready': spacePressed && !panning }"
				@pointerdown.capture="startPan"
				@pointermove="movePan"
				@pointerup="stopPan"
				@pointercancel="stopPan"
				@wheel="handleWheel"
			>
				<div v-if="loading" class="absolute inset-0 animate-pulse bg-surface-2" />
				<div class="flex min-h-full min-w-full">
					<div class="editor-canvas m-auto shrink-0">
						<canvas ref="canvasElement" />
					</div>
				</div>
			</div>
		</div>
	</NewModal>
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
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
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
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
	}

	&:focus-visible::-moz-range-thumb {
		box-shadow: 0 0 0 0.25rem var(--color-brand-shadow);
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

.editor-canvas :deep(.canvas-container) {
	flex: none;
	box-shadow:
		0 0 0 1px rgba(255, 255, 255, 0.14),
		0 18px 48px rgba(0, 0, 0, 0.38);
}

.editor-viewport.is-pan-ready :deep(.upper-canvas) {
	cursor: grab !important;
}

.editor-viewport.is-panning :deep(.upper-canvas) {
	cursor: grabbing !important;
}
</style>
