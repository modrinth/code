<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { injectImageViewerEditor } from '#ui/providers/image-viewer-editor'

import Editor from './editor.vue'
import type { ImageViewerEditorMode } from './image-viewer-editor-types'
import type {
	ImageViewerEditorData,
	ImageViewerEditorItem,
	ImageViewerEditorSavePayload,
} from './types'

const MAX_CACHED_ITEMS = 5

const props = withDefaults(
	defineProps<{
		items: ImageViewerEditorItem[]
		editor?: 'enabled' | 'disabled'
		saving?: boolean
	}>(),
	{
		editor: 'disabled',
		saving: false,
	},
)

const emit = defineEmits<{
	show: [item: ImageViewerEditorItem, index: number]
	hide: []
	navigate: [item: ImageViewerEditorItem, index: number, direction: 'next' | 'previous']
	save: [payload: ImageViewerEditorSavePayload]
}>()

const activeId = ref<string | null>(null)
const mode = ref<ImageViewerEditorMode>('view')
const titleContrast = ref<'dark' | 'light'>('light')
const descriptionContrast = ref<'dark' | 'light'>('light')
const closeAfterEditing = ref(false)
const editorComponent = ref<InstanceType<typeof Editor>>()
const titleElement = ref<HTMLElement>()
const descriptionElement = ref<HTMLElement>()
const context = injectImageViewerEditor(null)
const itemDataCache = new Map<string, Promise<ImageViewerEditorData>>()
const itemImageCache = new Map<string, HTMLImageElement>()
let headingContrastTimer: ReturnType<typeof setTimeout> | undefined

const activeIndex = computed(() => props.items.findIndex((item) => item.id === activeId.value))
const activeItem = computed(() => props.items[activeIndex.value] ?? null)
const canEdit = computed(
	() =>
		props.editor === 'enabled' &&
		Boolean(context?.loadEditorData) &&
		Boolean(activeItem.value?.editorSource),
)

watch(activeItem, (item) => {
	if (!item && activeId.value !== null) hide()
})

function getItemCacheKey(item: ImageViewerEditorItem) {
	return [item.id, item.src, item.editorSource?.id, item.editorSource?.path].join('\0')
}

function loadItemData(item: ImageViewerEditorItem): Promise<ImageViewerEditorData> {
	const key = getItemCacheKey(item)
	const cached = itemDataCache.get(key)
	if (cached) {
		itemDataCache.delete(key)
		itemDataCache.set(key, cached)
		return cached
	}

	const promise = (async () => {
		if (item.editorSource && context) return await context.loadEditorData(item.editorSource)
		const response = await fetch(item.src)
		if (!response.ok) throw new Error(`Could not load image: ${response.statusText}`)
		return { source: await response.blob() }
	})()

	itemDataCache.set(key, promise)
	while (itemDataCache.size > MAX_CACHED_ITEMS) {
		const oldestKey = itemDataCache.keys().next().value
		if (oldestKey === undefined) break
		itemDataCache.delete(oldestKey)
	}
	void promise.catch(() => {
		if (itemDataCache.get(key) === promise) itemDataCache.delete(key)
	})
	return promise
}

function preloadItemImage(item: ImageViewerEditorItem) {
	const key = getItemCacheKey(item)
	const cached = itemImageCache.get(key)
	if (cached) {
		itemImageCache.delete(key)
		itemImageCache.set(key, cached)
		return
	}

	const image = new Image()
	image.src = item.src
	itemImageCache.set(key, image)
	while (itemImageCache.size > MAX_CACHED_ITEMS) {
		const oldestKey = itemImageCache.keys().next().value
		if (oldestKey === undefined) break
		itemImageCache.delete(oldestKey)
	}
	void image.decode().catch(() => undefined)
}

function preloadItemsAround(index: number) {
	if (!props.items.length) return
	const indexes = new Set([
		index,
		(index - 1 + props.items.length) % props.items.length,
		(index + 1) % props.items.length,
	])
	for (const itemIndex of indexes) {
		const item = props.items[itemIndex]
		if (item) {
			preloadItemImage(item)
			void loadItemData(item).catch(() => undefined)
		}
	}
}

function show(index: number) {
	const item = props.items[index]
	if (!item) return
	cancelHeadingContrastUpdate()
	preloadItemsAround(index)
	if (activeId.value === null) {
		titleContrast.value = 'light'
		descriptionContrast.value = 'light'
		context?.onShow?.()
	}
	activeId.value = item.id
	mode.value = 'view'
	closeAfterEditing.value = false
	emit('show', item, index)
}

async function edit(index: number) {
	show(index)
	await beginEditing(true)
}

async function beginEditing(closeOnCancel = false) {
	if (!canEdit.value || props.saving) return
	closeAfterEditing.value = closeOnCancel
	mode.value = 'edit'
	await nextTick()
}

function finishEditing() {
	if (props.saving) return
	if (closeAfterEditing.value) {
		hide()
	} else {
		mode.value = 'view'
	}
}

function hide() {
	if (activeId.value === null || props.saving) return
	cancelHeadingContrastUpdate()
	activeId.value = null
	mode.value = 'view'
	closeAfterEditing.value = false
	itemDataCache.clear()
	itemImageCache.clear()
	context?.onHide?.()
	emit('hide')
}

function navigate(offset: number, direction: 'next' | 'previous') {
	if (mode.value !== 'view' || props.items.length < 2) return
	cancelHeadingContrastUpdate()
	const index = (activeIndex.value + offset + props.items.length) % props.items.length
	preloadItemsAround(index)
	activeId.value = props.items[index].id
	emit('navigate', props.items[index], index, direction)
}

function updateHeadingContrast() {
	cancelHeadingContrastUpdate()
	headingContrastTimer = setTimeout(() => {
		headingContrastTimer = undefined
		if (titleElement.value) {
			titleContrast.value = editorComponent.value?.getTextContrast(titleElement.value) ?? 'light'
		}
		if (descriptionElement.value) {
			descriptionContrast.value =
				editorComponent.value?.getTextContrast(descriptionElement.value) ?? 'light'
		}
	}, 120)
}

function cancelHeadingContrastUpdate() {
	if (headingContrastTimer === undefined) return
	clearTimeout(headingContrastTimer)
	headingContrastTimer = undefined
}

function next() {
	navigate(1, 'next')
}

function previous() {
	navigate(-1, 'previous')
}

async function markSavedAndView(itemId?: string) {
	editorComponent.value?.markSaved()
	mode.value = 'view'
	closeAfterEditing.value = false
	await nextTick()
	if (itemId && props.items.some((item) => item.id === itemId)) activeId.value = itemId
}

function handleKeydown(event: KeyboardEvent) {
	if (!activeItem.value || mode.value === 'edit' || document.querySelector('.modal-root')) return
	if (event.key === 'Escape') {
		event.preventDefault()
		hide()
	} else if (event.key === 'ArrowLeft') {
		event.preventDefault()
		previous()
	} else if (event.key === 'ArrowRight') {
		event.preventDefault()
		next()
	}
}

onMounted(() => document.addEventListener('keydown', handleKeydown))
onBeforeUnmount(() => {
	document.removeEventListener('keydown', handleKeydown)
	cancelHeadingContrastUpdate()
	itemDataCache.clear()
	itemImageCache.clear()
	if (activeId.value !== null) context?.onHide?.()
})

defineExpose({ show, edit, hide, next, previous, markSavedAndView })
</script>

<template>
	<Teleport to="body">
		<div
			v-if="activeItem"
			class="fixed inset-0 z-[110] overflow-hidden bg-black/95 text-white"
			role="dialog"
			aria-modal="true"
			:aria-label="activeItem.title || activeItem.alt"
			@click.self="mode === 'view' && hide()"
		>
			<header
				v-if="activeItem.title || activeItem.description"
				class="absolute inset-x-6 top-[calc(var(--top-bar-height,3rem)_+_1.5rem)] z-10 min-w-0"
				@click.stop
			>
				<div class="w-fit min-w-0 max-w-full">
					<h2
						v-if="activeItem.title"
						ref="titleElement"
						class="m-0 max-w-[min(42rem,70vw)] truncate text-base font-semibold leading-snug transition-colors duration-200 ease-out"
						:class="titleContrast === 'dark' ? 'text-gray-950' : 'text-white'"
					>
						{{ activeItem.title }}
					</h2>
					<p
						v-if="activeItem.description"
						ref="descriptionElement"
						class="mb-0 mt-1 max-w-[min(42rem,70vw)] truncate text-xs leading-snug opacity-70 transition-colors duration-200 ease-out"
						:class="descriptionContrast === 'dark' ? 'text-gray-950' : 'text-white'"
					>
						{{ activeItem.description }}
					</p>
				</div>
			</header>

			<Editor
				ref="editorComponent"
				:item="activeItem"
				:mode="mode"
				:index="activeIndex"
				:count="items.length"
				:can-edit="canEdit"
				:saving="saving"
				:load-data="loadItemData"
				@close="hide"
				@edit="beginEditing"
				@next="next"
				@previous="previous"
				@cancel="finishEditing"
				@save="emit('save', $event)"
				@image-ready="updateHeadingContrast"
			>
				<template #actions>
					<slot name="actions" :item="activeItem" :index="activeIndex" :hide="hide" />
				</template>
			</Editor>
		</div>
	</Teleport>
</template>
