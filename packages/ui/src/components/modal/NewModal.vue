<template>
	<Teleport to="body">
		<div v-if="open" class="modal-root">
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
						noblur: effectiveNoblur,
					},
					computedFade,
				]"
				@click="() => (closeOnClickOutside && closable ? hide() : {})"
			/>
			<div class="modal-container" :class="{ shown: visible }">
				<div
					ref="modalBodyRef"
					role="dialog"
					aria-modal="true"
					:aria-labelledby="headerId"
					class="modal-body flex flex-col bg-bg-raised rounded-2xl border border-solid border-surface-5"
					v-bind="$attrs"
					@keydown="handleKeyDown"
				>
					<div
						v-if="!hideHeader"
						data-tauri-drag-region
						class="grid grid-cols-[auto_min-content] items-center gap-4 p-6 border-solid border-0 border-b-[1px] border-surface-5 max-w-full"
					>
						<div class="flex text-wrap break-words items-center gap-3 min-w-0">
							<slot name="title">
								<span v-if="header" :id="headerId" class="text-2xl font-semibold text-contrast">
									{{ header }}
								</span>
							</slot>
						</div>
						<ButtonStyled v-if="closable" circular>
							<button
								v-tooltip="closeLabel"
								:aria-label="closeLabel"
								:disabled="disableClose"
								@click="hide"
							>
								<XIcon aria-hidden="true" />
							</button>
						</ButtonStyled>
					</div>

					<ButtonStyled
						v-if="props.mergeHeader && closable"
						class="absolute top-4 right-4 z-10"
						circular
					>
						<button
							v-tooltip="closeLabel"
							:aria-label="closeLabel"
							:disabled="disableClose"
							@click="hide"
						>
							<XIcon aria-hidden="true" />
						</button>
					</ButtonStyled>

					<div v-if="scrollable" class="relative flex-1 min-h-0 flex flex-col">
						<Transition
							enter-active-class="transition-all duration-200 ease-out"
							enter-from-class="opacity-0 max-h-0"
							enter-to-class="opacity-100 max-h-6"
							leave-active-class="transition-all duration-200 ease-in"
							leave-from-class="opacity-100 max-h-6"
							leave-to-class="opacity-0 max-h-0"
						>
							<div
								v-if="showTopFade"
								class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-6 bg-gradient-to-b from-bg-raised to-transparent"
							/>
						</Transition>

						<div
							ref="scrollContainer"
							:class="[
								'flex-1 min-h-0',
								props.noPadding ? '' : 'overflow-y-auto p-6 !pb-1 sm:pb-6',
								{ 'pt-12': props.mergeHeader && closable && !props.noPadding },
							]"
							:style="props.noPadding ? {} : { maxHeight: maxContentHeight }"
							@scroll="checkScrollState"
						>
							<slot> You just lost the game.</slot>
						</div>

						<Transition
							enter-active-class="transition-all duration-200 ease-out"
							enter-from-class="opacity-0 max-h-0"
							enter-to-class="opacity-100 max-h-6"
							leave-active-class="transition-all duration-200 ease-in"
							leave-from-class="opacity-100 max-h-6"
							leave-to-class="opacity-0 max-h-0"
						>
							<div
								v-if="showBottomFade"
								class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-6 bg-gradient-to-t from-bg-raised to-transparent"
							/>
						</Transition>
					</div>

					<div
						v-else
						:class="[
							props.noPadding ? '' : 'overflow-y-auto p-6',
							{ 'pt-12': props.mergeHeader && closable && !props.noPadding },
						]"
					>
						<slot> You just lost the game.</slot>
					</div>

					<div v-if="$slots.actions" class="p-4 pt-0">
						<slot name="actions" />
					</div>
				</div>
			</div>
		</div>
	</Teleport>
</template>

<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import { computed, nextTick, onUnmounted, ref } from 'vue'

import { useVIntl } from '../../composables/i18n'
import { useModalStack } from '../../composables/modal-stack'
import { useScrollIndicator } from '../../composables/scroll-indicator'
import { injectModalBehavior } from '../../providers'
import { commonMessages } from '../../utils/common-messages'
import ButtonStyled from '../base/ButtonStyled.vue'

const { formatMessage } = useVIntl()

const modalBehavior = injectModalBehavior(null)
const {
	push: pushModal,
	pop: popModal,
	isTopmost: isTopmostModal,
	stackSize: modalStackSize,
} = useModalStack()

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
		/** Removes padding from the content area. Useful for edge-to-edge layouts. */
		noPadding?: boolean
		/** Max width for the modal (e.g., '460px', '600px'). Defaults to '60rem'. */
		maxWidth?: string
		/** Width for the modal body (e.g., '460px', '600px'). */
		width?: string
		/** Disables all close actions (close button, ESC key, click outside). */
		disableClose?: boolean
	}>(),
	{
		type: true,
		noblur: undefined,
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
		noPadding: false,
		maxWidth: undefined,
		width: undefined,
		disableClose: false,
	},
)

const effectiveNoblur = computed(() => props.noblur ?? modalBehavior?.noblur.value ?? false)

const computedFade = computed(() => {
	if (props.fade) return props.fade
	if (props.danger) return 'danger'
	return 'standard'
})

const modalId = `modal-${Math.random().toString(36).slice(2, 9)}`
const headerId = `${modalId}-header`
const closeLabel = computed(() => formatMessage(commonMessages.closeButton))

const open = ref(false)
const visible = ref(false)
const stackDepth = ref(0)
const modalBodyRef = ref<HTMLElement | null>(null)
let previousFocusEl: Element | null = null

const scrollContainer = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState } = useScrollIndicator(scrollContainer)

const FOCUSABLE_SELECTOR =
	'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'

function getFocusableElements(): HTMLElement[] {
	if (!modalBodyRef.value) return []
	return Array.from(modalBodyRef.value.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR))
}

function show(event?: MouseEvent) {
	props.onShow?.()
	const wasEmpty = modalStackSize() === 0
	stackDepth.value = modalStackSize()
	open.value = true
	previousFocusEl = document.activeElement
	pushModal()
	if (wasEmpty) modalBehavior?.onShow?.()

	document.body.style.overflow = 'hidden'
	window.addEventListener('keydown', handleWindowKeyDown)
	window.addEventListener('mousedown', updateMousePosition)
	if (event) {
		updateMousePosition(event)
	} else {
		mouseX.value = Math.round(window.innerWidth / 2)
		mouseY.value = Math.round(window.innerHeight / 2)
	}
	setTimeout(() => {
		visible.value = true
		nextTick(() => {
			const focusable = getFocusableElements()
			if (focusable.length > 0) {
				focusable[0].focus()
			} else {
				modalBodyRef.value?.focus()
			}
		})
	}, 50)
}

function hide() {
	if (props.disableClose) return
	props.onHide?.()
	visible.value = false
	popModal()
	if (modalStackSize() === 0) {
		modalBehavior?.onHide?.()
		document.body.style.overflow = ''
	}
	window.removeEventListener('keydown', handleWindowKeyDown)
	window.removeEventListener('mousedown', updateMousePosition)
	if (previousFocusEl instanceof HTMLElement) {
		previousFocusEl.focus()
	}
	previousFocusEl = null
	setTimeout(() => {
		open.value = false
	}, 300)
}

defineExpose({
	show,
	hide,
	checkScrollState,
})

const mouseX = ref(0)
const mouseY = ref(0)

const MODAL_STACK_BASE_Z = 100
const stackZBase = computed(() => MODAL_STACK_BASE_Z + stackDepth.value * 10)
const stackOverlayZ = computed(() => stackZBase.value + 19)
const stackTauriZ = computed(() => stackZBase.value + 20)
const stackContainerZ = computed(() => stackZBase.value + 21)
const resolvedMaxWidth = computed(() => props.maxWidth ?? '60rem')
const resolvedWidth = computed(() => props.width ?? 'fit-content')
const mouseXOffset = computed(() => `calc((-50vw + ${mouseX.value}px) / 16)`)
const mouseYOffset = computed(() => `calc((-50vh + ${mouseY.value}px) / 16)`)

function updateMousePosition(event: { clientX: number; clientY: number }) {
	mouseX.value = event.clientX
	mouseY.value = event.clientY
}

onUnmounted(() => {
	if (open.value) {
		popModal()
		window.removeEventListener('keydown', handleWindowKeyDown)
		window.removeEventListener('mousedown', updateMousePosition)
		if (modalStackSize() === 0) {
			document.body.style.overflow = ''
			modalBehavior?.onHide?.()
		}
	}
})

function handleWindowKeyDown(event: KeyboardEvent) {
	if (props.closeOnEsc && event.key === 'Escape' && props.closable) {
		if (!isTopmostModal()) return
		hide()
		mouseX.value = Math.round(window.innerWidth / 2)
		mouseY.value = Math.round(window.innerHeight / 2)
	}
}

function handleKeyDown(event: KeyboardEvent) {
	if (event.key === 'Tab') {
		const focusable = getFocusableElements()
		if (focusable.length === 0) return

		const first = focusable[0]
		const last = focusable[focusable.length - 1]

		if (event.shiftKey) {
			if (document.activeElement === first) {
				event.preventDefault()
				last.focus()
			}
		} else {
			if (document.activeElement === last) {
				event.preventDefault()
				first.focus()
			}
		}
	}
}

defineOptions({
	inheritAttrs: false,
})
</script>

<style lang="scss" scoped>
.tauri-overlay {
	position: fixed;
	visibility: hidden;
	top: 0;
	left: 0;
	width: 100%;
	height: 100px;
	z-index: v-bind(stackTauriZ);

	&.shown {
		opacity: 1;
		visibility: visible;
	}
}

.modal-overlay {
	position: fixed;
	inset: -5rem;
	z-index: v-bind(stackOverlayZ);
	opacity: 0;
	visibility: hidden;
	pointer-events: none;
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
		pointer-events: auto;
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
	z-index: v-bind(stackContainerZ);
	visibility: hidden;
	pointer-events: none;
	transform: translate(v-bind(mouseXOffset), v-bind(mouseYOffset));
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
		max-width: min(v-bind(resolvedMaxWidth), calc(100% - 2 * var(--gap-lg)));
		overflow-y: hidden;
		overflow-x: hidden;
		width: v-bind(resolvedWidth);
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
