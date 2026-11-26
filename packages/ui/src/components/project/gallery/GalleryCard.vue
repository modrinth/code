<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CalendarIcon, EditIcon, StarIcon, TrashIcon } from '@modrinth/assets'

import { useFormatDateTime } from '../../../composables/format-date-time'
import { useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import IconButton from '../../base/buttons/IconButton.vue'
import FullImage from '../../base/FullImage.vue'
import { galleryCardMessages, galleryEditorMessages } from './messages'

defineProps<{
	image: Pick<Labrinth.Projects.v2.GalleryImage, 'url' | 'title' | 'description' | 'created'> & {
		raw_url?: string
		featured?: boolean
	}
	editable?: boolean
	actionsDisabled?: boolean
	heading?: 'h2' | 'h3'
}>()

const emit = defineEmits<{ preview: []; edit: []; remove: [] }>()
const { formatMessage } = useVIntl()
const formatDate = useFormatDateTime({ year: 'numeric', month: 'long', day: 'numeric' })
</script>

<template>
	<article
		class="group/gallery-card relative flex min-w-0 flex-col rounded-2xl border border-solid border-surface-5 bg-surface-3"
	>
		<slot name="thumbnail-overlay" />
		<button
			type="button"
			class="custom-focus-indicator group/preview block w-full cursor-pointer rounded-t-2xl border-0 bg-transparent p-0 focus-visible:outline-none"
			@click="emit('preview')"
		>
			<FullImage
				:src="image.url || 'https://cdn.modrinth.com/placeholder-banner.svg'"
				:raw-src="image.raw_url"
				:alt="image.title || formatMessage(galleryCardMessages.imageAlt)"
				class="block aspect-video w-full rounded-t-2xl object-cover transition-[filter] hover:brightness-75 group-focus-visible/preview:brightness-75"
			/>
			<span
				v-if="image.featured"
				v-tooltip="formatMessage(galleryEditorMessages.banner)"
				class="absolute right-2 top-2 flex size-9 items-center justify-center text-brand"
			>
				<StarIcon aria-hidden="true" class="size-5 fill-current" />
				<span class="sr-only">{{ formatMessage(galleryEditorMessages.banner) }}</span>
			</span>
		</button>
		<footer class="flex flex-1 flex-col gap-0.5 px-3 pt-2 pb-2">
			<div class="flex flex-col gap-3 [overflow-wrap:anywhere] empty:hidden pt-1">
				<div v-if="image.title || image.description" class="flex flex-col gap-1">
					<component
						:is="heading ?? 'h3'"
						v-if="image.title"
						class="m-0 text-base font-semibold text-contrast"
					>
						{{ image.title }}
					</component>
					<p v-if="image.description" class="m-0 text-sm text-secondary">
						{{ image.description }}
					</p>
				</div>
				<slot name="details" />
			</div>
			<div class="mt-auto flex flex-wrap items-center gap-2 min-h-8">
				<div class="flex min-h-8 items-center gap-1 text-sm text-secondary relative top-px">
					<CalendarIcon aria-hidden="true" class="size-4 shrink-0 mb-px" />
					<span class="sr-only">{{ formatMessage(galleryCardMessages.dateCreated) }}</span>
					{{ formatDate(image.created) }}
				</div>
				<div v-if="editable" class="ml-auto flex shrink-0 gap-0.5">
					<IconButton
						v-tooltip="formatMessage(commonMessages.editButton)"
						type="quiet"
						size="sm"
						:label="formatMessage(commonMessages.editButton)"
						:disabled="actionsDisabled"
						@click="emit('edit')"
					>
						<EditIcon aria-hidden="true" />
					</IconButton>
					<IconButton
						v-tooltip="formatMessage(commonMessages.removeButton)"
						type="quiet"
						size="sm"
						:label="formatMessage(commonMessages.removeButton)"
						:disabled="actionsDisabled"
						@click="emit('remove')"
					>
						<TrashIcon aria-hidden="true" />
					</IconButton>
				</div>
			</div>
		</footer>
	</article>
</template>
