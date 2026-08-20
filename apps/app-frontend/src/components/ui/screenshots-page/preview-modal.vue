<template>
	<Teleport to="body">
		<div
			v-if="activeItem"
			class="expanded-image-modal"
			role="dialog"
			aria-modal="true"
			:aria-label="activeItem.title || activeItem.alt"
			@click="hide"
		>
			<div class="content">
				<img
					class="image"
					:class="{ 'zoomed-in': zoomedIn }"
					:src="activeItem.src"
					:alt="activeItem.alt"
					@click.stop
				/>

				<div class="floating" @click.stop>
					<div v-if="activeItem.title || activeItem.description" class="text">
						<h2 v-if="activeItem.title">{{ activeItem.title }}</h2>
						<p v-if="activeItem.description">{{ activeItem.description }}</p>
					</div>
					<div class="controls">
						<div class="buttons">
							<IconButton :label="formatMessage(messages.close)" class="close" @click="hide">
								<XIcon aria-hidden="true" />
							</IconButton>
							<slot name="actions" :item="activeItem" :index="activeIndex" :hide="hide" />
							<IconButton :label="formatMessage(messages.toggleZoom)" @click="zoomedIn = !zoomedIn">
								<ExpandIcon v-if="!zoomedIn" aria-hidden="true" />
								<ContractIcon v-else aria-hidden="true" />
							</IconButton>
							<IconButton
								v-if="items.length > 1"
								:label="formatMessage(messages.previous)"
								@click="previous"
							>
								<LeftArrowIcon aria-hidden="true" />
							</IconButton>
							<IconButton
								v-if="items.length > 1"
								:label="formatMessage(messages.next)"
								@click="next"
							>
								<RightArrowIcon aria-hidden="true" />
							</IconButton>
						</div>
					</div>
				</div>
			</div>
		</div>
	</Teleport>
</template>

<script setup lang="ts">
import { ContractIcon, ExpandIcon, LeftArrowIcon, RightArrowIcon, XIcon } from '@modrinth/assets'
import { defineMessages, IconButton, useVIntl } from '@modrinth/ui'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { release_ads_window_hold, take_ads_window_hold } from '@/helpers/ads.js'

type ImagePreviewItem = {
	id: string
	src: string
	alt: string
	title?: string
	description?: string
}

const props = defineProps<{
	items: ImagePreviewItem[]
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	close: { id: 'app.image-preview.close', defaultMessage: 'Close' },
	toggleZoom: { id: 'app.image-preview.toggle-zoom', defaultMessage: 'Toggle zoom' },
	previous: { id: 'app.image-preview.previous', defaultMessage: 'Previous image' },
	next: { id: 'app.image-preview.next', defaultMessage: 'Next image' },
})

const emit = defineEmits<{
	show: [item: ImagePreviewItem, index: number]
	hide: []
	navigate: [item: ImagePreviewItem, index: number, direction: 'next' | 'previous']
}>()

const activeId = ref<string | null>(null)
const activeIndex = computed(() => props.items.findIndex((item) => item.id === activeId.value))
const activeItem = computed(() => props.items[activeIndex.value] ?? null)
const zoomedIn = ref(false)
let adsWindowHold = false

watch(activeItem, (item) => {
	if (!item && activeId.value !== null) hide()
})

function show(index: number) {
	const item = props.items[index]
	if (!item) return

	if (!adsWindowHold) {
		adsWindowHold = true
		take_ads_window_hold()
	}
	activeId.value = item.id
	zoomedIn.value = false
	emit('show', item, index)
}

function hide() {
	if (activeId.value === null) return
	activeId.value = null
	zoomedIn.value = false
	if (adsWindowHold) {
		adsWindowHold = false
		release_ads_window_hold()
	}
	emit('hide')
}

function navigate(offset: number, direction: 'next' | 'previous') {
	if (props.items.length < 2) return
	const index = (activeIndex.value + offset + props.items.length) % props.items.length
	activeId.value = props.items[index].id
	zoomedIn.value = false
	emit('navigate', props.items[index], index, direction)
}

function next() {
	navigate(1, 'next')
}

function previous() {
	navigate(-1, 'previous')
}

function keyListener(event: KeyboardEvent) {
	if (!activeItem.value || document.querySelector('.modal-root')) return

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

onMounted(() => document.addEventListener('keydown', keyListener))
onUnmounted(() => {
	document.removeEventListener('keydown', keyListener)
	if (adsWindowHold) release_ads_window_hold()
})

defineExpose({ show, hide, next, previous })
</script>

<style scoped lang="scss">
.expanded-image-modal {
	position: fixed;
	z-index: 110;
	overflow: auto;
	inset: 0;
	width: 100%;
	height: 100%;
	background-color: rgb(0 0 0 / 70%);
	display: flex;
	justify-content: center;
	align-items: center;

	.content {
		--controls-safe-area: 5rem;

		position: relative;
		width: calc(100% - 2 * var(--gap-lg));
		height: calc(100% - 2 * var(--gap-lg));

		.image {
			position: absolute;
			left: 50%;
			top: 50%;
			transform: translate(-50%, -50%);
			max-width: 100%;
			max-height: 100%;
			border-radius: var(--radius-lg);

			&.zoomed-in {
				object-fit: cover;
				top: calc((100% - var(--controls-safe-area)) / 2);
				width: auto;
				height: calc(100% - var(--controls-safe-area));
				max-width: 100%;
			}
		}

		.floating {
			position: absolute;
			left: 50%;
			transform: translateX(-50%);
			bottom: var(--gap-md);
			display: flex;
			flex-direction: column;
			align-items: center;
			gap: var(--gap-md);
			transition: opacity 0.25s ease-in-out;
			opacity: 1;
			padding: 2rem 2rem 0;

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
					color: var(--dark-color-base);
					font-size: 1.25rem;
					text-align: center;
					margin: 0;
				}

				p {
					color: var(--dark-color-base);
					margin: 0;
				}
			}

			.controls {
				background-color: var(--color-raised-bg);
				padding: var(--gap-md);
				border-radius: var(--radius-md);
				transition:
					opacity 0.25s ease-in-out,
					transform 0.25s ease-in-out;
			}
		}
	}
}

.buttons {
	display: flex;
	gap: 0.5rem;
}
</style>
