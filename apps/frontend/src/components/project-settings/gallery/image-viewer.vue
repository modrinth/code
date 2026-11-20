<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ContractIcon,
	ExpandIcon,
	ExternalIcon,
	LeftArrowIcon,
	RightArrowIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonLink,
	commonMessages,
	galleryCardMessages,
	IconButton,
	useVIntl,
} from '@modrinth/ui'
import { useEventListener } from '@vueuse/core'
import { computed, useTemplateRef } from 'vue'

import { messages } from './messages'

defineProps<{
	item: Pick<Labrinth.Projects.v2.GalleryImage, 'title' | 'description'> & { raw_url?: string }
	hasMultipleImages: boolean
}>()
const emit = defineEmits<{ close: []; previous: []; next: [] }>()
const zoomedIn = defineModel<boolean>('zoomedIn', { required: true })
const { formatMessage } = useVIntl()

const viewer = useTemplateRef<HTMLElement>('viewer')
const ownerDocument = computed(() => viewer.value?.ownerDocument)
useEventListener(ownerDocument, 'keydown', (event) => {
	event.preventDefault()
	if (event.key === 'Escape') {
		emit('close')
	} else if (event.key === 'ArrowLeft') {
		event.stopPropagation()
		emit('previous')
	} else if (event.key === 'ArrowRight') {
		event.stopPropagation()
		emit('next')
	}
})
</script>

<template>
	<div ref="viewer" class="expanded-image-modal" @click="emit('close')">
		<div class="content">
			<img
				class="image"
				:class="{ 'zoomed-in': zoomedIn }"
				:src="item.raw_url ? item.raw_url : 'https://cdn.modrinth.com/placeholder-banner.svg'"
				:alt="item.title || formatMessage(galleryCardMessages.imageAlt)"
				@click.stop
			/>

			<div class="floating" @click.stop>
				<div class="text">
					<h2 v-if="item.title">
						{{ item.title }}
					</h2>
					<p v-if="item.description">
						{{ item.description }}
					</p>
				</div>
				<div class="controls">
					<div class="flex gap-2">
						<IconButton
							:label="formatMessage(commonMessages.closeButton)"
							class="close"
							@click="emit('close')"
						>
							<XIcon aria-hidden="true" />
						</IconButton>
						<ButtonLink
							class="open !w-9 !rounded-full !px-0"
							target="_blank"
							:href="
								item.raw_url ? item.raw_url : 'https://cdn.modrinth.com/placeholder-banner.svg'
							"
						>
							<ExternalIcon aria-hidden="true" />
						</ButtonLink>
						<IconButton :label="formatMessage(messages.toggleZoom)" @click="zoomedIn = !zoomedIn">
							<ExpandIcon v-if="!zoomedIn" aria-hidden="true" />
							<ContractIcon v-else aria-hidden="true" />
						</IconButton>
						<IconButton
							v-if="hasMultipleImages"
							:label="formatMessage(messages.previousImage)"
							class="previous"
							@click="emit('previous')"
						>
							<LeftArrowIcon aria-hidden="true" />
						</IconButton>
						<IconButton
							v-if="hasMultipleImages"
							:label="formatMessage(messages.nextImage)"
							class="next"
							@click="emit('next')"
						>
							<RightArrowIcon aria-hidden="true" />
						</IconButton>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<style lang="scss" scoped>
.expanded-image-modal {
	position: fixed;
	z-index: 20;
	overflow: auto;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	background-color: #000000;
	background-color: rgba(0, 0, 0, 0.7);
	display: flex;
	justify-content: center;
	align-items: center;

	.content {
		position: relative;
		width: calc(100vw - 2 * var(--spacing-card-lg));
		height: calc(100vh - 2 * var(--spacing-card-lg));

		.image {
			position: absolute;
			left: 50%;
			top: 50%;
			transform: translate(-50%, -50%);
			max-width: calc(100vw - 2 * var(--spacing-card-lg));
			max-height: calc(100vh - 2 * var(--spacing-card-lg));
			border-radius: var(--size-rounded-card);

			&.zoomed-in {
				object-fit: cover;
				width: auto;
				height: calc(100vh - 2 * var(--spacing-card-lg));
				max-width: calc(100vw - 2 * var(--spacing-card-lg));
			}
		}
		.floating {
			position: absolute;
			left: 50%;
			transform: translateX(-50%);
			bottom: var(--spacing-card-md);
			display: flex;
			flex-direction: column;
			align-items: center;
			gap: var(--spacing-card-sm);
			transition: opacity 0.25s ease-in-out;
			opacity: 1;
			padding: 2rem 2rem 0 2rem;

			&:not(&:hover) {
				opacity: 0.4;
				.text {
					transform: translateY(2.5rem) scale(0.8);
					opacity: 0;
				}
				.controls {
					transform: translateY(0.25rem) scale(0.9);
				}
			}

			.text {
				display: flex;
				flex-direction: column;
				max-width: 40rem;
				transition:
					opacity 0.25s ease-in-out,
					transform 0.25s ease-in-out;
				text-shadow: 1px 1px 10px #000000d4;
				margin-bottom: 0.25rem;
				gap: 0.5rem;

				h2 {
					color: var(--dark-color-text-dark);
					font-size: 1.25rem;
					text-align: center;
					margin: 0;
				}

				p {
					color: var(--dark-color-text);
					margin: 0;
				}
			}
			.controls {
				background-color: var(--color-raised-bg);
				padding: var(--spacing-card-md);
				border-radius: var(--size-rounded-card);
				transition:
					opacity 0.25s ease-in-out,
					transform 0.25s ease-in-out;
			}
		}
	}
}
</style>
