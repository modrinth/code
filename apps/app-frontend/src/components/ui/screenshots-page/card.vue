<script lang="ts"></script>

<script setup lang="ts">
import { KeyboardSensor, PointerSensor, useDraggable } from '@dnd-kit/vue'
import { CheckIcon, ClipboardCopyIcon, EditIcon, MoreHorizontalIcon } from '@modrinth/assets'
import {
	defineMessages,
	IconButton,
	useDebugLogger,
	useFormatDateTime,
	useVIntl,
} from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import type { InstanceScreenshot } from '@/helpers/instance'

const props = defineProps<{
	screenshot: InstanceScreenshot
	selectionKey: string
	selected: boolean
	selectionActive: boolean
	activeDragged: boolean
	canDrag: boolean
	showInstanceName: boolean
	highlighted: boolean
	copied: boolean
}>()

const emit = defineEmits<{
	(e: 'activate', event: MouseEvent | KeyboardEvent): void
	(e: 'copy' | 'edit' | 'toggle-selection'): void
	(e: 'more', event: MouseEvent): void
}>()

const card = ref<HTMLElement>()
const image = ref<HTMLImageElement>()
const imageReady = ref(false)
const { formatMessage } = useVIntl()
const debugImage = useDebugLogger('Screenshots:Card')
const formatTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
let loadStartedAt = performance.now()
let loadGeneration = 0
const messages = defineMessages({
	select: { id: 'app.screenshots.select', defaultMessage: 'Select {name}' },
	deselect: { id: 'app.screenshots.deselect', defaultMessage: 'Deselect {name}' },
	copy: { id: 'app.screenshots.copy', defaultMessage: 'Copy image' },
	copied: { id: 'app.screenshots.copied', defaultMessage: 'Copied' },
	edit: { id: 'app.screenshots.edit', defaultMessage: 'Edit screenshot' },
	moreActions: { id: 'app.screenshots.more-actions', defaultMessage: 'More actions' },
})

const sensors = [
	PointerSensor.configure({
		preventActivation: () => false,
	}),
	KeyboardSensor,
]

useDraggable({
	id: computed(() => `screenshot:${props.selectionKey}`),
	element: card,
	disabled: computed(() => !props.canDrag),
	sensors,
	data: computed(() => ({
		selectionKey: props.selectionKey,
		instanceId: props.screenshot.instance_id,
	})),
})

function activate(event: MouseEvent | KeyboardEvent) {
	if (event instanceof KeyboardEvent) {
		if (event.target !== event.currentTarget || (event.key !== 'Enter' && event.key !== ' ')) {
			return
		}
		event.preventDefault()
	}
	emit('activate', event)
}

async function markImageLoaded() {
	const loadedImage = image.value
	const loadedUrl = props.screenshot.url
	const generation = loadGeneration
	if (!loadedImage) return

	try {
		await loadedImage.decode()
	} catch {
		if (!loadedImage.complete || loadedImage.naturalWidth === 0) return
	}

	await new Promise<void>((resolve) => {
		window.requestAnimationFrame(() => window.requestAnimationFrame(() => resolve()))
	})

	if (
		generation !== loadGeneration ||
		image.value !== loadedImage ||
		props.screenshot.url !== loadedUrl
	) {
		return
	}

	const wasReady = imageReady.value
	imageReady.value = true
	if (!wasReady) {
		debugImage('image loaded', {
			id: props.screenshot.id,
			fileName: props.screenshot.file_name,
			url: props.screenshot.url,
			loadDurationMs: performance.now() - loadStartedAt,
			naturalWidth: image.value?.naturalWidth,
			naturalHeight: image.value?.naturalHeight,
		})
	}
}

function markImageFailed(event: Event) {
	debugImage('image failed', {
		id: props.screenshot.id,
		fileName: props.screenshot.file_name,
		url: props.screenshot.url,
		loadDurationMs: performance.now() - loadStartedAt,
		event,
	})
}

onMounted(() => {
	debugImage('mounted', {
		id: props.screenshot.id,
		fileName: props.screenshot.file_name,
		url: props.screenshot.url,
		complete: image.value?.complete,
	})
	if (image.value?.complete && image.value.naturalWidth > 0) markImageLoaded()
})

onBeforeUnmount(() => {
	loadGeneration += 1
	debugImage('unmounted', {
		id: props.screenshot.id,
		fileName: props.screenshot.file_name,
		url: props.screenshot.url,
		loaded: imageReady.value,
	})
})

watch(
	() => props.screenshot.url,
	(url, previousUrl) => {
		loadGeneration += 1
		loadStartedAt = performance.now()
		imageReady.value = false
		debugImage('source changed', {
			id: props.screenshot.id,
			fileName: props.screenshot.file_name,
			previousUrl,
			url,
		})
	},
)
</script>

<template>
	<article
		ref="card"
		role="button"
		tabindex="0"
		class="group relative isolate aspect-video min-w-0 cursor-pointer overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-2 p-0 text-left shadow-sm transition-[filter] hover:brightness-110 focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
		:class="{
			'!border-contrast brightness-110': selected,
			'!border-brand ring-2 ring-brand animate-pulse': highlighted,
			'opacity-50': activeDragged,
			'cursor-grab active:cursor-grabbing': canDrag,
		}"
		data-screenshot-card
		:data-screenshot-id="screenshot.id"
		:data-selection-key="selectionKey"
		:aria-label="
			selectionActive
				? formatMessage(selected ? messages.deselect : messages.select, {
						name: screenshot.file_name,
					})
				: screenshot.file_name
		"
		:aria-pressed="selectionActive ? selected : undefined"
		@click="activate"
		@contextmenu.prevent.stop="emit('more', $event)"
		@keydown="activate"
	>
		<button
			type="button"
			class="selection-button group/selection absolute right-0.5 top-0 z-[3] flex size-[50px] cursor-pointer items-start justify-center border-0 bg-transparent p-0 pt-4"
			:aria-label="
				formatMessage(selected ? messages.deselect : messages.select, {
					name: screenshot.file_name,
				})
			"
			:aria-pressed="selected"
			@click.stop="emit('toggle-selection')"
		>
			<span
				class="relative flex size-6 items-center justify-center rounded-full opacity-0 transition-opacity duration-200 ease-out group-hover:opacity-100 group-focus-within:opacity-100 group-hover/selection:brightness-125"
				:class="
					selected ? 'border-0 !opacity-100' : 'border-2 border-solid border-primary bg-transparent'
				"
			>
				<span v-if="selected" class="absolute inset-0 rounded-full bg-contrast" />
				<CheckIcon v-if="selected" class="relative size-4 invert [stroke-width:3]" />
			</span>
		</button>
		<img
			ref="image"
			:src="screenshot.url"
			:alt="screenshot.file_name"
			loading="eager"
			decoding="async"
			draggable="false"
			class="screenshot-card-fade absolute inset-0 z-[1] h-full w-full object-cover"
			:class="imageReady ? 'opacity-100' : 'opacity-0'"
			@load="markImageLoaded"
			@error="markImageFailed"
		/>
		<div
			aria-hidden="true"
			class="pointer-events-none absolute inset-0 bg-button-bg"
			:class="{ 'animate-pulse': !imageReady }"
		/>
		<div
			class="absolute inset-x-0 bottom-0 z-[2] flex items-end justify-between gap-2 bg-gradient-to-t from-surface-1 to-transparent p-3 pt-[120px] text-contrast opacity-0 transition-opacity duration-200 group-hover:opacity-100 group-focus-within:opacity-100"
		>
			<div class="min-w-0">
				<div v-tooltip="screenshot.file_name" class="truncate text-sm font-semibold">
					{{ screenshot.file_name }}
				</div>
				<div class="truncate text-xs text-secondary">
					{{ showInstanceName ? screenshot.instance_name : formatTime(screenshot.created_at) }}
				</div>
			</div>
			<div
				v-if="!selectionActive"
				class="flex shrink-0 translate-y-1 gap-1 opacity-0 transition group-hover:translate-y-0 group-hover:opacity-100 group-focus-within:translate-y-0 group-focus-within:opacity-100"
				@click.stop
			>
				<IconButton
					v-tooltip="formatMessage(messages.edit)"
					:label="formatMessage(messages.edit)"
					type="quiet"
					class="bg-surface-2 text-contrast hover:bg-surface-3"
					@click="emit('edit')"
				>
					<EditIcon />
				</IconButton>
				<IconButton
					v-tooltip="formatMessage(copied ? messages.copied : messages.copy)"
					:label="formatMessage(copied ? messages.copied : messages.copy)"
					type="quiet"
					class="bg-surface-2 text-contrast hover:bg-surface-3"
					@click="emit('copy')"
				>
					<CheckIcon v-if="copied" class="text-green" />
					<ClipboardCopyIcon v-else />
				</IconButton>
				<IconButton
					v-tooltip="formatMessage(messages.moreActions)"
					:label="formatMessage(messages.moreActions)"
					type="quiet"
					class="bg-surface-2 text-contrast hover:bg-surface-3"
					@click="emit('more', $event)"
				>
					<MoreHorizontalIcon />
				</IconButton>
			</div>
		</div>
	</article>
</template>

<style scoped>
.screenshot-card-fade {
	transition: opacity 350ms ease-in-out;
}
</style>
