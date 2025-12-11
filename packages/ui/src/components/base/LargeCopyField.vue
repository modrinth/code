<script setup lang="ts">
import { CheckIcon, CopyIcon } from '@modrinth/assets';
import { ref } from 'vue';

const props = defineProps<{
	text: string
}>()

const copying = ref(false)

function copyText() {
	if (!copying.value) {
		copying.value = true
		setTimeout(() => (copying.value = false), 2000)
	}
}
</script>
<template>
	<button
		class="transition-all appearance-none border-0 rounded-xl text-start p-3 grid grid-cols-[1fr_auto] items-center gap-2"
		:class="{
			'bg-surface-4 hover:bg-surface-5 active:scale-[0.98]': !copying,
			'bg-brand-highlight': copying,
		}"
		:disabled="copying"
		@click="copyText"
	>
		<span class="text-contrast font-semibold">{{ text }}</span>
		<span class="text-primary icon-swapper">
			<Transition name="icon-swap-up">
				<CheckIcon v-if="copying" />
			</Transition>
			<Transition name="icon-swap-down">
				<CopyIcon v-if="!copying" />
			</Transition>
		</span>
	</button>
</template>
<style scoped>
.icon-swapper {
	display: grid;
	place-content: center;

	> * {
		grid-area: 1 / 1;
	}
}

.icon-swap-up-enter-active, .icon-swap-up-leave-active, .icon-swap-down-enter-active, .icon-swap-down-leave-active {
	transition: opacity 0.2s ease-in-out, translate 0.2s ease-in-out;
}

.icon-swap-up-enter-from {
	opacity: 0;
	translate: 0 -100%;
}

.icon-swap-up-leave-to {
	opacity: 0;
	translate: 0 -100%;
}

.icon-swap-down-enter-from {
	opacity: 0;
	translate: 0 100%;
}

.icon-swap-down-leave-to {
	opacity: 0;
	translate: 0 100%;
}
</style>
