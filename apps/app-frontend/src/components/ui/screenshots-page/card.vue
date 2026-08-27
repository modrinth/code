<script lang="ts"></script>

<script setup lang="ts">
import { KeyboardSensor, PointerSensor, useDraggable } from '@dnd-kit/vue'
import { CheckIcon, ClipboardCopyIcon, EditIcon, MoreHorizontalIcon } from '@modrinth/assets'
import { defineMessages, IconButton, useFormatDateTime, useVIntl } from '@modrinth/ui'
import { computed, onMounted, ref, watch } from 'vue'

import type { InstanceScreenshot } from '@/helpers/instance'
const loadedScreenshotUrls = new Set<string>()

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
const loaded = ref(loadedScreenshotUrls.has(props.screenshot.url))
const { formatMessage } = useVIntl()
const formatTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
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

function markImageLoaded() {
	loadedScreenshotUrls.add(props.screenshot.url)
	loaded.value = true
}

onMounted(() => {
	if (image.value?.complete && image.value.naturalWidth > 0) markImageLoaded()
})

watch(
	() => props.screenshot.url,
	(url) => {
		loaded.value = loadedScreenshotUrls.has(url)
	},
)
</script>

<template>
	<article
		ref="card"
		role="button"
		tabindex="0"
		class="group relative aspect-video min-w-0 cursor-pointer overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-2 p-0 text-left shadow-sm transition-[filter] hover:brightness-110 focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
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
			class="selection-button group/selection absolute right-0.5 top-0 z-[2] flex size-[50px] cursor-pointer items-start justify-center border-0 bg-transparent p-0 pt-4"
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
		<div v-if="!loaded" class="absolute inset-0 animate-pulse bg-surface-3" />
		<img
			ref="image"
			:src="screenshot.url"
			:alt="screenshot.file_name"
			loading="lazy"
			draggable="false"
			class="h-full w-full object-cover transition duration-200"
			:class="loaded ? 'opacity-100' : 'opacity-0'"
			@load="markImageLoaded"
		/>
		<div
			class="absolute inset-x-0 bottom-0 flex items-end justify-between gap-2 bg-gradient-to-t from-surface-1 to-transparent p-3 pt-[120px] text-contrast opacity-0 transition-opacity duration-200 group-hover:opacity-100 group-focus-within:opacity-100"
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
