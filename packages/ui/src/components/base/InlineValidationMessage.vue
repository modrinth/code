<template>
	<Transition name="validation-message">
		<div v-if="message" aria-live="polite">
			<div class="flex w-full items-start gap-1 text-red">
				<XCircleIcon class="shrink-0 mt-[3px]" aria-hidden="true" />
				<span class="min-w-0 break-words">{{ message }}</span>
			</div>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import { XCircleIcon } from '@modrinth/assets'

defineProps<{
	message?: string
}>()
</script>

<style scoped>
.validation-message-enter-active,
.validation-message-leave-active {
	transition:
		grid-template-rows 150ms ease,
		opacity 150ms ease,
		transform 150ms ease;
}

.validation-message-enter-from,
.validation-message-leave-to {
	grid-template-rows: 0fr;
	opacity: 0;
	transform: translateY(-0.25rem);
}

@media (prefers-reduced-motion: reduce) {
	.validation-message-enter-active,
	.validation-message-leave-active {
		transition: none;
	}
}
</style>
