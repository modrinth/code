<template>
	<GalleryGrid
		:images="filteredGallery"
		class="grid w-full grid-cols-[repeat(auto-fill,minmax(min(100%,20rem),1fr))] gap-4"
		@preview="expandImage"
	/>
	<ImageViewerEditor
		ref="galleryViewer"
		:items="galleryViewerItems"
		editor="disabled"
		@navigate="trackGalleryNavigation"
	>
		<template #actions="{ item }">
			<Button
				type="quiet"
				class="!w-9 !rounded-full !p-0"
				aria-label="Open image in new tab"
				@click="openUrl(item.src)"
			>
				<ExternalIcon aria-hidden="true" />
			</Button>
		</template>
	</ImageViewerEditor>
</template>

<script setup>
import { ExternalIcon } from '@modrinth/assets'
import { Button, GalleryGrid, ImageViewerEditor } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'

const MC_SERVER_BANNER_NAME = '__mc_server_banner__'

const props = defineProps({
	project: {
		type: Object,
		default: () => ({}),
	},
})

const filteredGallery = computed(
	() => props.project.gallery?.filter((img) => img.title !== MC_SERVER_BANNER_NAME) ?? [],
)

const galleryViewer = ref()
const galleryViewerItems = computed(() =>
	filteredGallery.value.map((image) => ({
		id: image.url,
		src: image.raw_url ?? 'https://cdn.modrinth.com/placeholder-banner.svg',
		alt: image.title || 'Gallery image',
		title: image.title,
		description: image.description,
	})),
)

function expandImage(index) {
	galleryViewer.value?.show(index)
	trackEvent('GalleryImageExpand', {
		project_id: props.project.id,
		url: filteredGallery.value[index].url,
	})
}

function trackGalleryNavigation(item, _index, direction) {
	trackEvent(direction === 'next' ? 'GalleryImageNext' : 'GalleryImagePrevious', {
		project_id: props.project.id,
		url: item.id,
	})
}
</script>
