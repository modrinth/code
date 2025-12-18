<template>
	<div
		v-if="open"
		:style="`${mouseX !== -1 ? `--_mouse-x: ${mouseX};` : ''} ${mouseY !== -1 ? `--_mouse-y: ${mouseY};` : ''}`"
	>
		<div
			:class="{ shown: visible }"
			class="tauri-overlay"
			data-tauri-drag-region
			@click="() => (closeOnClickOutside && closable ? hide() : {})"
		/>
		<div
			:class="[
				'modal-overlay',
				{
					shown: visible,
					noblur: props.noblur,
				},
				computedFade,
			]"
			@click="() => (closeOnClickOutside && closable ? hide() : {})"
		/>
		<div class="modal-container experimental-styles-within" :class="{ shown: visible }">
			<div class="modal-body flex flex-col bg-bg-raised rounded-2xl">
				<div
					v-if="!hideHeader"
					data-tauri-drag-region
					class="grid grid-cols-[auto_min-content] items-center gap-12 p-6 border-solid border-0 border-b-[1px] border-divider max-w-full"
				>
					<div class="flex text-wrap break-words items-center gap-3 min-w-0">
						<slot name="title">
							<span v-if="header" class="text-lg font-extrabold text-contrast">
								{{ header }}
							</span>
						</slot>
					</div>
					<ButtonStyled v-if="closable" circular>
						<button v-tooltip="'Close'" aria-label="Close" @click="hide">
							<XIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
				</div>

				<ButtonStyled
					v-if="props.mergeHeader && closable"
					class="absolute top-4 right-4 z-10"
					circular
				>
					<button v-tooltip="'Close'" aria-label="Close" @click="hide">
						<XIcon aria-hidden="true" />
					</button>
				</ButtonStyled>

				<div v-if="scrollable" class="relative">
					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-24"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-24"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showTopFade"
							class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-24 bg-gradient-to-b from-bg-raised to-transparent"
						/>
					</Transition>

					<div
						ref="scrollContainer"
						:class="[
							'overflow-y-auto p-6 !pb-1 sm:pb-6',
							{ 'pt-12': props.mergeHeader && closable },
						]"
						:style="{ maxHeight: maxContentHeight }"
						@scroll="checkScrollState"
					>
						<slot> You just lost the game.</slot>
					</div>

					<Transition
						enter-active-class="transition-all duration-200 ease-out"
						enter-from-class="opacity-0 max-h-0"
						enter-to-class="opacity-100 max-h-24"
						leave-active-class="transition-all duration-200 ease-in"
						leave-from-class="opacity-100 max-h-24"
						leave-to-class="opacity-0 max-h-0"
					>
						<div
							v-if="showBottomFade"
							class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-24 bg-gradient-to-t from-bg-raised to-transparent"
						/>
					</Transition>
				</div>

				<div v-else :class="['overflow-y-auto p-6', { 'pt-12': props.mergeHeader && closable }]">
					<slot> You just lost the game.</slot>
				</div>

				<div v-if="$slots.actions" class="p-6 pt-0">
					<slot name="actions" />
				</div>
			</div>
		</div>
	</div>
	<div v-else></div>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'

import { useScrollIndicator } from '../../composables/scroll-indicator'
import ButtonStyled from '../base/ButtonStyled.vue'

const props = withDefaults(
	defineProps<{
		noblur?: boolean
		closable?: boolean
		/** @deprecated Use `fade="danger"` instead */
		danger?: boolean
		fade?: 'standard' | 'warning' | 'danger'
		closeOnEsc?: boolean
		closeOnClickOutside?: boolean
		warnOnClose?: boolean
		header?: string
		hideHeader?: boolean
		onHide?: () => void
		onShow?: () => void
		mergeHeader?: boolean
		scrollable?: boolean
		maxContentHeight?: string
	}>(),
	{
		type: true,
		closable: true,
		danger: false,
		fade: undefined,
		closeOnClickOutside: true,
		closeOnEsc: true,
		warnOnClose: false,
		header: undefined,
		hideHeader: false,
		onHide: () => {},
		onShow: () => {},
		mergeHeader: false,
		// TODO: migrate all modals to use scrollable and remove this prop
		scrollable: false,
		maxContentHeight: '70vh',
	},
)

const computedFade = computed(() => {
	if (props.fade) return props.fade
	if (props.danger) return 'danger'
	return 'standard'
})

const open = ref(false)
const visible = ref(false)

const scrollContainer = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState } = useScrollIndicator(scrollContainer)

function show(event?: MouseEvent) {
	props.onShow?.()
	open.value = true

	document.body.style.overflow = 'hidden'
	window.addEventListener('mousedown', updateMousePosition)
	window.addEventListener('keydown', handleKeyDown)
	if (event) {
		updateMousePosition(event)
	} else {
		mouseX.value = window.innerWidth / 2
		mouseY.value = window.innerHeight / 2
	}
	setTimeout(() => {
		visible.value = true
	}, 50)
}

function hide() {
	props.onHide?.()
	visible.value = false
	document.body.style.overflow = ''
	window.removeEventListener('mousedown', updateMousePosition)
	window.removeEventListener('keydown', handleKeyDown)
	setTimeout(() => {
		open.value = false
	}, 300)
}

defineExpose({
	show,
	hide,
	checkScrollState,
})

const mouseX = ref(-1)
const mouseY = ref(-1)

function updateMousePosition(event: { clientX: number; clientY: number }) {
	mouseX.value = event.clientX
	mouseY.value = event.clientY
}

function handleKeyDown(event: KeyboardEvent) {
	if (props.closeOnEsc && event.key === 'Escape' && props.closable) {
		hide()
		mouseX.value = window.innerWidth / 2
		mouseY.value = window.innerHeight / 2
	}
}
</script>

<style lang="scss" scoped>
.tauri-overlay {
	position: fixed;
	visibility: hidden;
	top: 0;
	left: 0;
	width: 100%;
	height: 100px;
	z-index: 20;

	&.shown {
		opacity: 1;
		visibility: visible;
	}
}

.modal-overlay {
	position: fixed;
	inset: -5rem;
	z-index: 19;
	opacity: 0;
	transition: all 0.2s ease-out;
	//transform: translate(
	//    calc((-50vw + var(--_mouse-x, 50vw) * 1px) / 2),
	//    calc((-50vh + var(--_mouse-y, 50vh) * 1px) / 2)
	//  )
	//  scaleX(0.8) scaleY(0.5);
	border-radius: 180px;
	//filter: blur(5px);

	// Fade variants
	&.standard {
		background: linear-gradient(to bottom, rgba(29, 48, 43, 0.52) 0%, rgba(14, 21, 26, 0.95) 100%);
	}

	&.warning {
		background: linear-gradient(to bottom, rgba(48, 38, 29, 0.52) 0%, rgba(26, 20, 14, 0.95) 100%);
	}

	&.danger {
		background: linear-gradient(to bottom, rgba(43, 18, 26, 0.52) 0%, rgba(49, 10, 15, 0.95) 100%);
	}

	@media (prefers-reduced-motion) {
		transition: none !important;
	}

	&.shown {
		opacity: 1;
		visibility: visible;
		backdrop-filter: blur(5px);
	}

	&.noblur {
		backdrop-filter: none;
		filter: none;
	}
}

.modrinth-parent__no-modal-blurs {
	.modal-overlay {
		backdrop-filter: none;
	}
}

.modal-container {
	position: fixed;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	display: flex;
	justify-content: center;
	align-items: center;
	z-index: 21;
	visibility: hidden;
	pointer-events: none;
	transform: translate(
		calc((-50vw + var(--_mouse-x, 50vw) * 1px) / 16),
		calc((-50vh + var(--_mouse-y, 50vh) * 1px) / 16)
	);
	transition: all 0.2s ease-out;

	&.shown {
		visibility: visible;
		transform: translate(0, 0);

		> .modal-body {
			opacity: 1;
			visibility: visible;
			scale: 1;
		}
	}

	> .modal-body {
		position: fixed;
		box-shadow: 4px 4px 26px 10px rgba(0, 0, 0, 0.08);
		max-height: calc(100% - 2 * var(--gap-lg));
		max-width: min(var(--_max-width, 60rem), calc(100% - 2 * var(--gap-lg)));
		overflow-y: hidden;
		overflow-x: hidden;
		width: fit-content;
		pointer-events: auto;
		scale: 0.97;

		visibility: hidden;
		opacity: 0;
		transition: all 0.2s ease-in-out;

		@media (prefers-reduced-motion) {
			transition: none !important;
		}

		@media screen and (max-width: 640px) {
			width: calc(100% - 2 * var(--gap-lg));
		}
	}
}
</style>
