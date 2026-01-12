<script setup lang="ts">
defineProps<{
	shown: boolean
}>()
</script>

<template>
	<Transition name="floating-action-bar" appear>
		<div v-if="shown" class="floating-action-bar fixed w-full z-10 left-0 p-4 bottom-0">
			<div
				class="flex items-center gap-2 rounded-2xl bg-bg-raised border-2 border-divider border-solid mx-auto max-w-[77rem] p-4"
			>
				<slot />
			</div>
		</div>
	</Transition>
</template>

<style scoped>
.floating-action-bar {
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
