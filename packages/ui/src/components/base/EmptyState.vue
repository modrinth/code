<template>
	<div
		class="relative mx-auto flex w-full max-w-[448px] flex-col items-center overflow-hidden px-4 text-center"
		:class="hidePattern ? 'py-16' : 'pb-[63px] pt-[160px]'"
	>
		<div
			v-if="!hidePattern"
			aria-hidden="true"
			class="dot-pattern pointer-events-none absolute left-1/2 top-[-31px] h-[478px] w-[415px] -translate-x-1/2"
		/>
		<component
			:is="icon"
			v-if="icon"
			class="relative size-[72px] stroke-[1.5px] text-primary"
			aria-hidden="true"
		/>
		<div class="relative flex flex-col items-center gap-2" :class="{ 'mt-6': icon }">
			<h2
				v-if="$slots.heading || heading"
				class="m-0 text-2xl font-semibold leading-8 text-contrast"
			>
				<slot name="heading">{{ heading }}</slot>
			</h2>
			<p v-if="$slots.description || description" class="m-0 text-base leading-6 text-primary">
				<slot name="description">{{ description }}</slot>
			</p>
		</div>
		<div v-if="$slots.actions" class="relative mt-6 flex flex-wrap justify-center gap-2">
			<slot name="actions" />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
	icon?: Component
	heading?: string
	description?: string
	hidePattern?: boolean
}>()
</script>

<style scoped>
.dot-pattern {
	background-image: radial-gradient(circle, #d9d9d9 0.5px, transparent 1px);
	background-size: 9px 9px;
	mask-image: radial-gradient(closest-side, black, transparent);
}

:is(.dark-mode, .oled-mode, .retro-mode) .dot-pattern {
	opacity: 0.1;
}
</style>
