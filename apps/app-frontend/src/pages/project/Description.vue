<template>
	<Card>
		<!-- eslint-disable-next-line vue/no-v-html -->
		<div ref="container" class="markdown-body" @click="onClick" v-html="renderedBody" />
	</Card>

	<Teleport to="body">
		<Transition name="video-overlay-fade">
			<div
				v-if="videoOpen"
				class="video-overlay-backdrop"
				@click="closeOverlay"
				@keydown.esc="closeOverlay"
			>
				<!-- The native video webview is positioned by Rust centered in this
				     backdrop. This region is just a sizing placeholder + close button. -->
				<div class="video-overlay-frame" @click.stop>
					<button
						type="button"
						class="video-overlay-close"
						aria-label="Close video"
						@click="closeOverlay"
					>
						<XIcon />
					</button>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<script setup>
import { XIcon } from '@modrinth/assets'
import { Card } from '@modrinth/ui'
import { renderHighlightedString } from '@modrinth/utils'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { closeVideoOverlay, openVideoOverlay } from '@/helpers/utils.js'

const props = defineProps({
	project: {
		type: Object,
		default: () => {},
	},
})

const YOUTUBE_EMBED = /^https?:\/\/(?:www\.)?youtube(?:-nocookie)?\.com\/embed\/([a-zA-Z0-9_-]{11})/

const videoOpen = ref(false)

// Inline YouTube iframes fail with Error 153 in the app (the tauri:// webview
// sends no Referer to the cross-origin frame). Swap each embed for a thumbnail
// facade that opens the video as a centered in-app overlay webview.
function facadeForIframe(iframe) {
	const src = iframe.getAttribute('src') ?? ''
	const match = src.match(YOUTUBE_EMBED)
	if (!match) {
		return null
	}

	const videoId = match[1]
	const facade = document.createElement('button')
	facade.type = 'button'
	facade.className = 'video-facade'
	facade.dataset.videoId = videoId
	facade.setAttribute('aria-label', 'Play YouTube video')
	facade.innerHTML =
		`<img loading="lazy" alt="" src="https://i.ytimg.com/vi/${videoId}/hqdefault.jpg" />` +
		`<span class="video-facade__play" aria-hidden="true"></span>`
	return facade
}

const renderedBody = computed(() => {
	const html = renderHighlightedString(props.project?.body ?? '')
	if (typeof document === 'undefined') {
		return html
	}

	const template = document.createElement('template')
	template.innerHTML = html
	for (const iframe of template.content.querySelectorAll('iframe')) {
		const facade = facadeForIframe(iframe)
		if (facade) {
			iframe.replaceWith(facade)
		}
	}
	return template.innerHTML
})

function onClick(event) {
	const facade = event.target.closest('.video-facade')
	if (!facade) {
		return
	}
	event.preventDefault()
	videoOpen.value = true
	openVideoOverlay(facade.dataset.videoId)
}

function closeOverlay() {
	videoOpen.value = false
	closeVideoOverlay()
}

// Keep the native webview in sync if the overlay is dismissed by navigation, etc.
watch(videoOpen, (open) => {
	if (!open) {
		closeVideoOverlay()
	}
})

function onKeydown(event) {
	if (event.key === 'Escape' && videoOpen.value) {
		closeOverlay()
	}
}
window.addEventListener('keydown', onKeydown)
onBeforeUnmount(() => {
	window.removeEventListener('keydown', onKeydown)
	if (videoOpen.value) {
		closeVideoOverlay()
	}
})
</script>

<script>
export default {
	name: 'Description',
}
</script>

<style scoped lang="scss">
.markdown-body :deep(.video-facade) {
	position: relative;
	display: block;
	width: 100%;
	max-width: 560px;
	aspect-ratio: 16 / 9;
	padding: 0;
	border: none;
	border-radius: var(--radius-md);
	overflow: hidden;
	cursor: pointer;
	background: var(--color-button-bg);

	img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.video-facade__play {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 68px;
		height: 48px;
		border-radius: 14px;
		background: rgba(0, 0, 0, 0.7);
		transition: background 0.15s ease-in-out;

		&::after {
			content: '';
			position: absolute;
			top: 50%;
			left: 52%;
			transform: translate(-50%, -50%);
			border-style: solid;
			border-width: 11px 0 11px 19px;
			border-color: transparent transparent transparent #fff;
		}
	}

	&:hover .video-facade__play,
	&:focus-visible .video-facade__play {
		background: var(--color-brand);
	}
}

.video-overlay-backdrop {
	position: fixed;
	inset: 0;
	z-index: 1000;
	display: flex;
	align-items: center;
	justify-content: center;
	background: rgba(0, 0, 0, 0.8);
}

// Matches the 80%/16:9 rect the Rust side uses for the native video webview,
// so the close button sits just outside its top-right corner.
.video-overlay-frame {
	position: relative;
	width: 80%;
	aspect-ratio: 16 / 9;
	max-height: 80%;
}

.video-overlay-close {
	position: absolute;
	top: -2.75rem;
	right: 0;
	display: flex;
	align-items: center;
	justify-content: center;
	width: 2.25rem;
	height: 2.25rem;
	padding: 0;
	border: none;
	border-radius: var(--radius-md);
	background: var(--color-button-bg);
	color: var(--color-contrast);
	cursor: pointer;

	svg {
		width: 1.25rem;
		height: 1.25rem;
	}

	&:hover {
		filter: brightness(1.25);
	}
}

.video-overlay-fade-enter-active,
.video-overlay-fade-leave-active {
	transition: opacity 0.15s ease-in-out;
}

.video-overlay-fade-enter-from,
.video-overlay-fade-leave-to {
	opacity: 0;
}
</style>
