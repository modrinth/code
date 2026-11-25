<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { GripVerticalIcon } from '@modrinth/assets'
import {
	GalleryCard,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useIsMutating, useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, shallowRef, watch } from 'vue'
import Draggable from 'vuedraggable'

import { galleryReorderMessages } from './messages'

type GalleryImage = Labrinth.Projects.v2.GalleryImage & { raw_url?: string }

interface GalleryOrderChange {
	url: string
	ordering: number
	previousOrdering: number
}

interface ReorderGalleryRequest {
	projectId: string
	changes: GalleryOrderChange[]
}

const props = defineProps<{
	projectId: string
	images: GalleryImage[]
	editable: boolean
	busy: boolean
	heading?: 'h2' | 'h3'
}>()
const emit = defineEmits<{
	preview: [index: number]
	edit: [index: number]
	remove: [url: string]
}>()
const reordering = defineModel<boolean>('reordering', { default: false })
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const orderedGallery = shallowRef<GalleryImage[]>([])
const dragStartGallery = shallowRef<GalleryImage[] | null>(null)

const mutationKey = computed(() => ['project', props.projectId, 'gallery-order'])
const activeReorders = useIsMutating(() => ({ mutationKey: mutationKey.value }))

const { mutate: reorderGallery, isPending } = useMutation({
	mutationKey,
	mutationFn: async ({ projectId, changes }: ReorderGalleryRequest) => {
		const saved: GalleryOrderChange[] = []
		try {
			// TODO: Replace these sequential requests with a bulk PATCH that saves the gallery order atomically.
			for (const change of changes) {
				await client.labrinth.projects_v2.editGalleryImage(projectId, change.url, {
					ordering: change.ordering,
				})
				saved.push(change)
			}
		} catch (error) {
			for (const change of saved.reverse()) {
				await client.labrinth.projects_v2
					.editGalleryImage(projectId, change.url, { ordering: change.previousOrdering })
					.catch(() => undefined)
			}
			throw error
		}
	},
	onSuccess: (_, { projectId, changes }) => {
		const newOrderings = new Map(changes.map(({ url, ordering }) => [url, ordering]))
		for (const version of ['v2', 'v3']) {
			queryClient.setQueryData<{ gallery?: GalleryImage[] }>(
				['project', version, projectId],
				(current) =>
					current?.gallery && {
						...current,
						gallery: current.gallery
							.map((image) => ({
								...image,
								ordering: newOrderings.get(image.url) ?? image.ordering,
							}))
							.sort((a, b) => a.ordering - b.ordering),
					},
			)
		}
	},
	onError: () => {
		addNotification({ type: 'error', title: formatMessage(galleryReorderMessages.reorderError) })
	},
	onSettled: async (_, _error, { projectId }) => {
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['project', 'v2', projectId] }),
			queryClient.invalidateQueries({ queryKey: ['project', 'v3', projectId] }),
		])
	},
})

const isReorderingGallery = computed(() => isPending.value || activeReorders.value > 0)

watch(
	() => isReorderingGallery.value || !!dragStartGallery.value,
	(value) => (reordering.value = value),
	{ immediate: true, flush: 'sync' },
)

const canReorderGallery = computed(
	() =>
		props.editable &&
		!props.busy &&
		!isReorderingGallery.value &&
		orderedGallery.value.length > 1 &&
		orderedGallery.value.every((image) => image.url),
)

watch(
	[() => props.images, isReorderingGallery],
	([images, isSaving]) => {
		if (!dragStartGallery.value && !isSaving) orderedGallery.value = [...images]
	},
	{ immediate: true },
)

function galleryItemKey(image: GalleryImage) {
	return image.url || image.created
}

function startGalleryDrag() {
	dragStartGallery.value = props.images
}

function finishGalleryDrag() {
	const beforeDrag = dragStartGallery.value
	dragStartGallery.value = null
	const current = props.images
	if (
		!beforeDrag ||
		beforeDrag.length !== current.length ||
		beforeDrag.some(
			(image, index) =>
				image.url !== current[index].url || image.ordering !== current[index].ordering,
		)
	) {
		orderedGallery.value = [...current]
		return
	}
	saveGalleryOrder()
}

function moveGalleryImage(event: KeyboardEvent, index: number) {
	const offsets: Record<string, number> = {
		ArrowLeft: -1,
		ArrowUp: -1,
		ArrowRight: 1,
		ArrowDown: 1,
	}
	const offset = offsets[event.key]
	if (!offset || !canReorderGallery.value || dragStartGallery.value) return
	event.preventDefault()
	const target = index + offset
	const image = orderedGallery.value[index]
	if (!image || target < 0 || target >= orderedGallery.value.length) return
	const reordered = [...orderedGallery.value]
	reordered.splice(index, 1)
	reordered.splice(target, 0, image)
	orderedGallery.value = reordered
	saveGalleryOrder()
}

function saveGalleryOrder() {
	const current = props.images
	if (
		!canReorderGallery.value ||
		orderedGallery.value.every((image, index) => image.url === current[index]?.url)
	) {
		orderedGallery.value = [...current]
		return
	}
	const orderings = current.map((image) => image.ordering)
	const hasUniqueOrderings = new Set(orderings).size === current.length
	const changes = orderedGallery.value
		.map((image, index) => ({
			url: image.url,
			previousOrdering: image.ordering,
			ordering: hasUniqueOrderings ? orderings[index] : index,
		}))
		.filter(({ previousOrdering, ordering }) => previousOrdering !== ordering)
	reorderGallery({ projectId: props.projectId, changes })
}

function selectImage(action: 'preview' | 'edit', image: GalleryImage) {
	const index = props.images.findIndex(
		(candidate) => galleryItemKey(candidate) === galleryItemKey(image),
	)
	if (index < 0) return
	if (action === 'preview') emit('preview', index)
	else emit('edit', index)
}
</script>

<template>
	<Draggable
		v-model="orderedGallery"
		:item-key="galleryItemKey"
		handle=".gallery-reorder-handle"
		:disabled="!canReorderGallery"
		:animation="250"
		:swap-threshold="1"
		:invert-swap="false"
		:force-fallback="true"
		:fallback-on-body="true"
		:fallback-tolerance="4"
		ghost-class="gallery-reorder-ghost"
		fallback-class="gallery-reorder-fallback"
		@start="startGalleryDrag"
		@end="finishGalleryDrag"
	>
		<template #item="{ element: image, index }">
			<GalleryCard
				:image="image"
				:editable="editable"
				:actions-disabled="busy || reordering || !image.url"
				:heading="heading"
				@preview="selectImage('preview', image)"
				@edit="selectImage('edit', image)"
				@remove="emit('remove', image.url)"
			>
				<template #thumbnail-overlay>
					<IconButton
						v-if="editable && orderedGallery.length > 1"
						:circular="false"
						:label="formatMessage(galleryReorderMessages.reorderImage)"
						:aria-disabled="!canReorderGallery"
						class="gallery-reorder-handle !absolute left-2 top-2 z-10 !cursor-grab !touch-none focus-visible:opacity-100 active:!cursor-grabbing group-hover/gallery-card:opacity-100 aria-disabled:!cursor-default [@media(hover:hover)]:opacity-0"
						@keydown="moveGalleryImage($event, index)"
					>
						<GripVerticalIcon aria-hidden="true" />
					</IconButton>
				</template>
				<template #details>
					<slot name="details" :image="image" />
				</template>
			</GalleryCard>
		</template>
	</Draggable>
</template>

<style scoped>
:global(.gallery-reorder-ghost) {
	opacity: 0.35;
}

:global(.gallery-reorder-fallback) {
	opacity: 0.9;
	pointer-events: none;
}
</style>
