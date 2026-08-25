<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { injectImageViewerEditor } from '#ui/providers/image-viewer-editor'

import Editor from './editor.vue'
import type { ImageViewerEditorMode } from './image-viewer-editor-types'
import type { ImageViewerEditorItem, ImageViewerEditorSavePayload } from './types'
import Viewer from './viewer.vue'

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
const editorComponent = ref<InstanceType<typeof Editor>>()
const context = injectImageViewerEditor(null)

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

function show(index: number) {
	const item = props.items[index]
	if (!item) return
	if (activeId.value === null) context?.onShow?.()
	activeId.value = item.id
	mode.value = 'view'
	emit('show', item, index)
}

async function edit(index: number) {
	show(index)
	await beginEditing()
}

async function beginEditing() {
	if (!canEdit.value || props.saving) return
	mode.value = 'edit'
	await nextTick()
}

function finishEditing() {
	if (!props.saving) mode.value = 'view'
}

function hide() {
	if (activeId.value === null || props.saving) return
	activeId.value = null
	mode.value = 'view'
	context?.onHide?.()
	emit('hide')
}

function navigate(offset: number, direction: 'next' | 'previous') {
	if (mode.value !== 'view' || props.items.length < 2) return
	const index = (activeIndex.value + offset + props.items.length) % props.items.length
	activeId.value = props.items[index].id
	emit('navigate', props.items[index], index, direction)
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
				<div class="min-w-0">
					<h2
						v-if="activeItem.title"
						class="m-0 max-w-[min(42rem,70vw)] truncate text-base font-semibold leading-snug text-white"
					>
						{{ activeItem.title }}
					</h2>
					<p
						v-if="activeItem.description"
						class="mb-0 mt-1 max-w-[min(42rem,70vw)] truncate text-xs leading-snug text-white/60"
					>
						{{ activeItem.description }}
					</p>
				</div>
			</header>

			<Viewer
				v-if="mode === 'view'"
				:item="activeItem"
				:index="activeIndex"
				:count="items.length"
				:can-edit="canEdit"
				@close="hide"
				@edit="beginEditing"
				@next="next"
				@previous="previous"
			>
				<template #actions>
					<slot name="actions" :item="activeItem" :index="activeIndex" :hide="hide" />
				</template>
			</Viewer>
			<Editor
				v-else
				ref="editorComponent"
				:item="activeItem"
				:saving="saving"
				@cancel="finishEditing"
				@save="emit('save', $event)"
			/>
		</div>
	</Teleport>
</template>
