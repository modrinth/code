<script setup lang="ts">
import { onUnmounted, ref, watch } from 'vue'

const props = defineProps<{
	shown: boolean
	ariaLabel?: string
}>()

const toolbarEl = ref<HTMLElement | null>(null)
const compact = ref(false)

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

let observer: ResizeObserver | null = null

watch(
	toolbarEl,
	(el) => {
		observer?.disconnect()
		if (!el) return
		observer = new ResizeObserver(() => {
			checkCompact()
		})
		observer.observe(el.parentElement!)
		checkCompact()
	},
	{ immediate: true },
)

watch(
	() => props.shown,
	(shown) => {
		document?.body.classList.toggle('floating-action-bar-shown', shown)
	},
	{ immediate: true },
)

onUnmounted(() => {
	observer?.disconnect()
	document?.body.classList.remove('floating-action-bar-shown')
})
</script>

<template>
	<Transition name="floating-action-bar" appear>
		<div
			v-if="shown"
			class="floating-action-bar drop-shadow-2xl fixed z-[21] p-4 bottom-0"
			aria-live="polite"
		>
			<div
				ref="toolbarEl"
				role="toolbar"
				:aria-label="ariaLabel"
				class="relative overflow-clip flex items-center gap-2 rounded-[20px] bg-surface-3 border border-surface-5 border-solid mx-auto max-w-[60vw] px-4 py-3 shadow-[0px_1px_3px_0px_rgba(0,0,0,0.3),0px_6px_10px_0px_rgba(0,0,0,0.15)]"
				:class="{ 'bar-compact': compact }"
			>
				<slot />
			</div>
		</div>
	</Transition>
</template>

<style scoped>
.floating-action-bar {
	left: var(--left-bar-width, 0px);
	right: var(--right-bar-width, 0px);
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
.intercom-lightweight-app-launcher {
	z-index: 9 !important;
}

.bar-compact .bar-label {
	display: none;
}

.bar-compact .cq-show-icon {
	display: block;
}
</style>
