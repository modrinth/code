<template>
	<div
		class="notification-stack"
		:class="{
			'has-sidebar': hasSidebar,
		}"
	>
		<TransitionGroup name="notification-stack-item">
			<slot />
		</TransitionGroup>
	</div>
</template>

<script setup lang="ts">
withDefaults(
	defineProps<{
		hasSidebar?: boolean
	}>(),
	{
		hasSidebar: false,
	},
)
</script>

<style scoped>
.notification-stack {
	position: fixed;
	top: calc(var(--top-bar-height, 3rem) + 1rem);
	right: 1rem;
	z-index: 200;
	display: flex;
	width: 360px;
	flex-direction: column;
	gap: 1rem;
}

.notification-stack.has-sidebar {
	right: calc(var(--right-bar-width, 0px) + 1rem);
}

@media screen and (max-width: 500px) {
	.notification-stack {
		right: 0.75rem;
		width: 360px;
	}
}

.notification-stack-item-enter-active,
.notification-stack-item-leave-active,
.notification-stack-item-move {
	transition: all 0.3s ease-in-out;
}

.notification-stack-item-enter-from,
.notification-stack-item-leave-to {
	opacity: 0;
	transform: translateX(100%) scale(0.95);
}
</style>
