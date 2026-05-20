<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

import { useModalStack } from '../../composables/modal-stack'
import { injectPageContext } from '../../providers'

const props = defineProps<{
	shown: boolean
	ariaLabel?: string
	belowModal?: boolean
	hideWhenModalOpen?: boolean
}>()

const INTERCOM_BUBBLE_GAP = 8

const barEl = ref<HTMLElement | null>(null)
const toolbarEl = ref<HTMLElement | null>(null)
const compact = ref(false)

const { stackCount } = useModalStack()
const pageContext = injectPageContext(null)
const shown = computed(() => props.shown && (!props.hideWhenModalOpen || stackCount.value === 0))
const intercomBubbleClearanceRequestId = Symbol('floating-action-bar')
const zIndex = computed(() => 100 + stackCount.value * 10 + 8 + (!props.belowModal ? 1 : 0))
const leftOffset = computed(
	() => pageContext?.floatingActionBarOffsets?.left.value ?? 'var(--left-bar-width, 0px)',
)
const rightOffset = computed(
	() => pageContext?.floatingActionBarOffsets?.right.value ?? 'var(--right-bar-width, 0px)',
)
const barStyle = computed(() => ({
	zIndex: zIndex.value,
	'--floating-action-bar-left-offset': leftOffset.value,
	'--floating-action-bar-right-offset': rightOffset.value,
}))

function checkCompact() {
	const el = toolbarEl.value
	if (!el) return

	const clone = el.cloneNode(true) as HTMLElement
	clone.classList.remove('bar-compact')
	clone.style.position = 'absolute'
	clone.style.visibility = 'hidden'
	clone.style.pointerEvents = 'none'
	clone.style.width = `${el.offsetWidth}px`

	el.parentElement!.appendChild(clone)
	const needsCompact = clone.offsetHeight > 70
	clone.remove()

	compact.value = needsCompact
}

function clearIntercomBubbleClearance() {
	pageContext?.intercomBubble?.requestVerticalClearance(intercomBubbleClearanceRequestId, null)
}

function updateIntercomBubbleClearance() {
	const intercomBubble = pageContext?.intercomBubble
	if (!intercomBubble) return

	if (typeof window === 'undefined' || !shown.value || !barEl.value || !toolbarEl.value) {
		clearIntercomBubbleClearance()
		return
	}

	const barRect = barEl.value.getBoundingClientRect()
	const toolbarRight = barRect.left + toolbarEl.value.offsetLeft + toolbarEl.value.offsetWidth
	const bubbleLeft =
		window.innerWidth - intercomBubble.horizontalPadding.value - intercomBubble.width.value

	if (toolbarRight + INTERCOM_BUBBLE_GAP <= bubbleLeft) {
		clearIntercomBubbleClearance()
		return
	}

	const barStyle = window.getComputedStyle(barEl.value)
	const bottomOffset = Number.parseFloat(barStyle.bottom) || 0
	intercomBubble.requestVerticalClearance(
		intercomBubbleClearanceRequestId,
		Math.ceil(bottomOffset + barEl.value.offsetHeight + INTERCOM_BUBBLE_GAP),
	)
}

function updateBodyState(isShown = shown.value) {
	if (typeof document === 'undefined') return

	document.body.classList.toggle('floating-action-bar-shown', isShown)
	if (!isShown) {
		clearIntercomBubbleClearance()
	}
}

let observer: ResizeObserver | null = null
let updateFrame: number | null = null

function scheduleIntercomBubbleClearanceUpdate() {
	if (typeof window === 'undefined') return
	if (updateFrame !== null) {
		window.cancelAnimationFrame(updateFrame)
	}

	updateFrame = window.requestAnimationFrame(() => {
		updateFrame = null
		updateIntercomBubbleClearance()
	})
}

watch(
	toolbarEl,
	(el) => {
		observer?.disconnect()
		if (!el) return
		observer = new ResizeObserver(() => {
			checkCompact()
			scheduleIntercomBubbleClearanceUpdate()
		})
		observer.observe(el.parentElement!)
		checkCompact()
		scheduleIntercomBubbleClearanceUpdate()
	},
	{ immediate: true },
)

watch(
	shown,
	async (isShown) => {
		await nextTick()
		updateBodyState(isShown)
		scheduleIntercomBubbleClearanceUpdate()
	},
	{ immediate: true },
)

watch(
	[
		shown,
		leftOffset,
		rightOffset,
		() => pageContext?.intercomBubble?.horizontalPadding.value,
		() => pageContext?.intercomBubble?.width.value,
	],
	() => scheduleIntercomBubbleClearanceUpdate(),
	{ immediate: true },
)

onMounted(() => {
	window.addEventListener('resize', scheduleIntercomBubbleClearanceUpdate)
	scheduleIntercomBubbleClearanceUpdate()
})

onUnmounted(() => {
	observer?.disconnect()
	window.removeEventListener('resize', scheduleIntercomBubbleClearanceUpdate)
	if (updateFrame !== null) {
		window.cancelAnimationFrame(updateFrame)
	}
	clearIntercomBubbleClearance()
	if (typeof document === 'undefined') return
	document.body.classList.remove('floating-action-bar-shown')
})
</script>

<template>
	<Teleport to="body">
		<Transition name="floating-action-bar" appear>
			<div
				v-if="shown"
				ref="barEl"
				class="floating-action-bar drop-shadow-2xl fixed p-4 bottom-0"
				:style="barStyle"
				aria-live="polite"
			>
				<div
					ref="toolbarEl"
					role="toolbar"
					:aria-label="ariaLabel"
					class="relative overflow-clip flex items-center gap-1.5 rounded-[20px] bg-surface-3 border border-surface-5 border-solid mx-auto max-w-[60vw] px-3 py-2.5 shadow-[0px_1px_3px_0px_rgba(0,0,0,0.3),0px_6px_10px_0px_rgba(0,0,0,0.15)]"
					:class="{ 'bar-compact': compact }"
				>
					<slot />
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<style scoped>
.floating-action-bar {
	left: var(--floating-action-bar-left-offset, var(--left-bar-width, 0px));
	right: var(--floating-action-bar-right-offset, var(--right-bar-width, 0px));
	transition: bottom 0.25s ease-in-out;
}

.floating-action-bar-enter-active {
	transition:
		transform 0.25s cubic-bezier(0.15, 1.4, 0.64, 0.96),
		opacity 0.25s cubic-bezier(0.15, 1.4, 0.64, 0.96);
}

.floating-action-bar-leave-active {
	transition:
		transform 0.25s ease,
		opacity 0.25s ease;
}

.floating-action-bar-enter-from {
	transform: scale(0.5) translateY(10rem);
	opacity: 0;
}

.floating-action-bar-leave-to {
	transform: scale(0.96) translateY(0.25rem);
	opacity: 0;
}

@media (any-hover: none) and (max-width: 640px) {
	.floating-action-bar {
		bottom: var(--size-mobile-navbar-height);
	}

	.expanded-mobile-nav .floating-action-bar {
		bottom: var(--size-mobile-navbar-height-expanded);
	}
}
</style>

<style>
.bar-compact .bar-label {
	display: none;
}

.bar-compact .cq-show-icon {
	display: block;
}
</style>
