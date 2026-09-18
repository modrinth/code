<template>
	<div>
		<ReviewPanel
			mode="inline"
			:target="{ kind: 'gallery' }"
			:disabled="isLoading || !!error"
			class="mb-2"
		>
		</ReviewPanel>
		<h2 class="m-0 text-lg font-semibold text-contrast">
			{{ formatMessage(messages.gallery) }}
		</h2>
		<p v-if="!selection" class="m-0 text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>
		<p v-else-if="isLoading" role="status" class="m-0">
			{{ formatMessage(messages.loading) }}
		</p>
		<div v-else-if="error" role="alert">
			<p class="m-0">{{ formatMessage(messages.loadError) }}</p>
			<Button @click="refresh">{{ formatMessage(messages.retry) }}</Button>
		</div>
		<p v-else-if="!gallery.length" class="m-0 text-secondary">
			{{ formatMessage(messages.emptyGallery) }}
		</p>
		<div v-else class="gallery-grid">
			<ReviewPanel
				mode="anchored"
				v-for="(item, index) in gallery"
				:key="item.url"
				as="article"
				trigger-placement="overlay"
				:target="{ kind: 'gallery-image', key: item.url }"
				class="flex min-w-0 flex-col overflow-hidden rounded-xl border border-solid border-surface-4"
			>
				<button
					type="button"
					class="cursor-zoom-in border-0 p-0"
					:aria-label="formatMessage(messages.openImage, { number: index + 1 })"
					@click="viewer?.show(index)"
				>
					<img
						:src="item.raw_url || item.url"
						:alt="item.name || formatMessage(messages.imageNumber, { number: index + 1 })"
						class="aspect-video w-full object-contain"
						loading="lazy"
					/>
				</button>
				<div class="flex flex-1 flex-col gap-2 p-3">
					<div class="flex flex-wrap justify-between gap-2 text-sm text-secondary">
						<span>{{ formatMessage(messages.imageNumber, { number: index + 1 }) }}</span>
						<span v-if="item.featured" class="text-brand">{{
							formatMessage(messages.featured)
						}}</span>
					</div>
					<h3 v-if="item.name" class="m-0 break-words text-lg font-semibold text-contrast">
						{{ item.name }}
					</h3>
					<p v-if="item.description" class="m-0 whitespace-pre-wrap break-words">
						{{ item.description }}
					</p>
					<time :datetime="item.created" class="mt-auto pt-2 text-sm text-secondary">{{
						formatMessage(messages.uploaded, {
							date: formatDateTime(item.created),
						})
					}}</time>
				</div>
			</ReviewPanel>
		</div>
		<ImageViewerEditor :key="projectId" ref="viewer" :items="viewerItems" editor="disabled" />
	</div>
</template>

<script setup lang="ts">
import { Button, ImageViewerEditor, useFormatDateTime, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { injectProjectReviewPageContext } from '~/providers/project-review'

import { projectReviewMessages as messages } from '../messages'
import ReviewPanel from '../review-panel/index.vue'

const { formatMessage } = useVIntl()
const formatDateTime = useFormatDateTime({
	dateStyle: 'long',
	timeStyle: 'short',
})
const { selection, projectId, gallery, isLoading, error, refresh } =
	injectProjectReviewPageContext()
const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
const viewerItems = computed(() =>
	gallery.value.map((item, index) => ({
		id: item.url,
		src: item.raw_url || item.url,
		alt: item.name || formatMessage(messages.imageNumber, { number: index + 1 }),
		title: item.name,
		description: item.description,
	})),
)
</script>

<style scoped>
.gallery-grid {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(min(100%, 20rem), 1fr));
	gap: 1rem;
}
</style>
