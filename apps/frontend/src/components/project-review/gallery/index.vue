<template>
	<div ref="interactionScope" class="flex h-full min-h-0 flex-col gap-2.5 overflow-hidden p-2">
		<ReviewPanel
			mode="inline"
			:target="{ kind: 'gallery' }"
			hide-title
			:disabled="isLoading || !!error"
			:interaction-scope="interactionScope"
			class="shrink-0"
		/>
		<div class="min-h-0 flex-1 overflow-y-auto overscroll-contain">
			<p v-if="!selection" class="m-0 text-secondary">No project selected</p>
			<p v-else-if="isLoading" role="status" class="m-0">Loading...</p>
			<div v-else-if="error" role="alert">
				<p class="m-0">Could not load project data</p>
				<Button @click="refresh">Retry</Button>
			</div>
			<p v-else-if="!gallery.length" class="m-0 text-secondary">No gallery images</p>
			<div
				v-else
				class="gallery-grid"
				:style="{ gridTemplateColumns: `repeat(${galleryColumns}, minmax(0, 1fr))` }"
			>
				<ReviewPanel
					v-for="(item, index) in gallery"
					:key="item.id ?? item.url"
					mode="anchored"
					:target="{ kind: 'gallery-image', key: imageReviewKey(item) ?? '' }"
					:disabled="imageReviewKey(item) === undefined"
					as="article"
					class="flex min-w-0 flex-col overflow-hidden rounded-xl border border-solid border-surface-4"
				>
					<button
						type="button"
						class="cursor-zoom-in border-0 p-0"
						:aria-label="`Open image ${index + 1}`"
						@click="viewer?.show(index)"
					>
						<img
							:src="item.raw_url || item.url"
							:alt="item.name || `Image ${index + 1}`"
							class="aspect-video w-full object-contain"
							loading="lazy"
						/>
					</button>
					<div class="flex flex-1 flex-col gap-1.5 p-3">
						<div class="flex flex-wrap items-center gap-2">
							<h3 class="m-0 min-w-0 flex-1 break-words text-lg font-semibold text-contrast">
								{{ item.name || `Image ${index + 1}` }}
							</h3>
							<TagItem
								v-if="item.featured"
								class="-my-0.5 !border-brand !bg-transparent !text-brand opacity-90"
							>
								Featured
							</TagItem>
						</div>
						<p v-if="item.description" class="m-0 whitespace-pre-wrap break-words">
							{{ item.description }}
						</p>
						<time
							:datetime="item.created"
							class="mt-auto flex items-center gap-1 pt-1 text-sm text-secondary"
						>
							<CalendarIcon aria-hidden="true" class="h-4 w-4 shrink-0" />
							{{ formatDate(item.created) }}
						</time>
					</div>
				</ReviewPanel>
			</div>
		</div>
		<ImageViewerEditor
			:key="projectId"
			ref="viewer"
			:items="viewerItems"
			editor="disabled"
			:pixelated="pixelated"
		>
			<template #actions="{ item, metadata }">
				<ImageViewerActions v-model:pixelated="pixelated" :src="item.src" :metadata="metadata" />
			</template>
		</ImageViewerEditor>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CalendarIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import { galleryImageReviewKey } from '@modrinth/moderation/src/data/issues/component-builders/gallery-images'
import { Button, ImageViewerEditor, TagItem } from '@modrinth/ui'
import { computed, ref, useTemplateRef } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import { createDateFormatter } from '../format'
import ImageViewerActions from '../image-viewer-actions.vue'
import ReviewPanel from '../review-panel/index.vue'

const settings = useModerationSettings()
const galleryColumns = computed(() => settings.value.get(moderationSettings.General.GalleryColumns))
const interactionScope = useTemplateRef<HTMLElement>('interactionScope')
const formatDate = createDateFormatter({
	year: 'numeric',
	month: 'long',
	day: 'numeric',
})
const { selection, projectId, gallery, isLoading, error, refresh, pixelated } =
	injectProjectReviewPageContext()
const { sendIssuesAsThreadMessage } = injectReviewSubmission()
const imageReviewKey = (image: Labrinth.Projects.v3.GalleryItem) =>
	galleryImageReviewKey(image, sendIssuesAsThreadMessage.value)
const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
const viewerItems = computed(() =>
	gallery.value.map((item, index) => ({
		id: item.url,
		src: item.raw_url || item.url,
		alt: item.name || `Image ${index + 1}`,
		title: item.name,
		description: item.description,
	})),
)
</script>

<style scoped>
.gallery-grid {
	display: grid;
	gap: 1rem;
}
</style>
