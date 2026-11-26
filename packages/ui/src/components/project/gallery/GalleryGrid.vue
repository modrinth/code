<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'

import GalleryCard from './GalleryCard.vue'

defineProps<{
	images: (Labrinth.Projects.v2.GalleryImage & { raw_url?: string })[]
}>()
const emit = defineEmits<{ preview: [index: number] }>()
</script>

<template>
	<div>
		<GalleryCard
			v-for="(image, index) in images"
			:key="image.url || image.created"
			:image="image"
			@preview="emit('preview', index)"
		/>
	</div>
</template>
