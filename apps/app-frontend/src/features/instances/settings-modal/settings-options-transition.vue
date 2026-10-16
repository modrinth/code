<script setup lang="ts">
defineProps<{
	show: boolean
}>()
</script>

<template>
	<Transition name="settings-options">
		<div v-if="show" class="settings-options" :inert="!show">
			<div>
				<slot />
			</div>
		</div>
	</Transition>
</template>

<style scoped>
.settings-options {
	display: grid;
	grid-template-rows: 1fr;
}

.settings-options-enter-active,
.settings-options-leave-active {
	transition:
		grid-template-rows 0.25s var(--ease-out-expo),
		opacity 0.2s ease-out;
}

.settings-options-enter-from,
.settings-options-leave-to {
	grid-template-rows: 0fr;
	opacity: 0;
}

.settings-options > div {
	min-height: 0;
}

.settings-options-enter-active > div,
.settings-options-leave-active > div {
	overflow: hidden;
}

@media (prefers-reduced-motion) {
	.settings-options-enter-active,
	.settings-options-leave-active {
		transition: none;
	}
}
</style>
