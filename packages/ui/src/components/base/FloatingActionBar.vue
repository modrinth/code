<script setup lang="ts">
import { onUnmounted, watch } from 'vue'

const props = defineProps<{
	shown: boolean
}>()

watch(
	() => props.shown,
	(shown) => {
		document?.body.classList.toggle('floating-action-bar-shown', shown)
	},
	{ immediate: true },
)

onUnmounted(() => {
	document?.body.classList.remove('floating-action-bar-shown')
})
</script>

<template>
	<Transition name="floating-action-bar" appear>
		<div v-if="shown" class="floating-action-bar drop-shadow-2xl fixed z-10 p-4 bottom-0">
			<div
				class="flex items-center gap-2 rounded-[20px] bg-surface-3 border border-surface-5 border-solid mx-auto max-w-[60vw] px-4 py-3 shadow-[0px_1px_3px_0px_rgba(0,0,0,0.3),0px_6px_10px_0px_rgba(0,0,0,0.15)]"
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
</style>
