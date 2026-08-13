<template>
	<div class="gallery">
		<Card v-for="(image, index) in filteredGallery" :key="image.url" class="gallery-item">
			<a @click="expandImage(image, index)">
				<img :src="image.url" :alt="image.title" class="gallery-image" />
			</a>
			<div class="gallery-body">
				<h3>{{ image.title }}</h3>
				{{ image.description }}
			</div>
			<span class="gallery-time">
				<CalendarIcon />
				{{ formatDate(new Date(image.created)) }}
			</span>
		</Card>
	</div>
	<ImagePreviewModal
		ref="imagePreviewModal"
		:items="previewItems"
		@navigate="trackPreviewNavigation"
	>
		<template #actions="{ item }">
			<ButtonLink
				class="open btn icon-only !w-9 !rounded-full !px-0"
				target="_blank"
				:href="item.src"
			>
				<ExternalIcon aria-hidden="true" />
			</ButtonLink>
		</template>
	</ImagePreviewModal>
</template>

<script setup>
import { CalendarIcon, ExternalIcon } from '@modrinth/assets'
import { ButtonLink, Card, useFormatDateTime } from '@modrinth/ui'
import { computed, ref } from 'vue'

import ImagePreviewModal from '@/components/ui/ImagePreviewModal.vue'
import { trackEvent } from '@/helpers/analytics'

const MC_SERVER_BANNER_NAME = '__mc_server_banner__'

const formatDate = useFormatDateTime({
	year: 'numeric',
	month: 'long',
	day: 'numeric',
})

const props = defineProps({
	project: {
		type: Object,
		default: () => ({}),
	},
})

const filteredGallery = computed(
	() => props.project.gallery?.filter((img) => img.title !== MC_SERVER_BANNER_NAME) ?? [],
)

const imagePreviewModal = ref()
const previewItems = computed(() =>
	filteredGallery.value.map((item) => ({
		id: item.url,
		src: item.raw_url || 'https://cdn.modrinth.com/placeholder-banner.svg',
		alt: item.title || 'gallery-image',
		title: item.title,
		description: item.description,
	})),
)

const expandImage = (item, index) => {
	imagePreviewModal.value?.show(index)

	trackEvent('GalleryImageExpand', {
		project_id: props.project.id,
		url: item.url,
	})
}

function trackPreviewNavigation(_item, index, direction) {
	trackEvent(direction === 'next' ? 'GalleryImageNext' : 'GalleryImagePrevious', {
		project_id: props.project.id,
		url: filteredGallery.value[index]?.url,
	})
}
</script>

<style scoped lang="scss">
.gallery {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
	width: 100%;
	gap: 1rem;
}

.gallery-item {
	padding: 0;
	overflow: hidden;
	margin: 0;
	display: flex;
	flex-direction: column;

	.gallery-image {
		width: 100%;
		aspect-ratio: 2/1;
		object-fit: cover;
		object-position: center;
	}

	.gallery-body {
		flex-grow: 1;
		padding: 1rem;
	}

	.gallery-time {
		padding: 0 1rem 1rem;
		vertical-align: center;
	}
}
</style>
