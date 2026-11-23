<template>
	<div
		class="mx-auto flex flex-col items-center text-center"
		:class="
			icon
				? 'relative w-full max-w-[448px] overflow-hidden px-4 pb-[63px] pt-[160px]'
				: 'max-w-full p-6'
		"
	>
		<template v-if="icon">
			<div
				aria-hidden="true"
				class="dot-pattern pointer-events-none absolute left-1/2 top-[-31px] h-[478px] w-[415px] -translate-x-1/2"
			/>
			<component
				:is="icon"
				class="relative size-[72px] stroke-[1.5px] text-primary"
				aria-hidden="true"
			/>
		</template>
		<slot v-else name="illustration">
			<component :is="illustration" v-if="illustration" class="h-[200px] w-auto max-w-full" />
		</slot>
		<div class="flex flex-col items-center" :class="icon ? 'relative mt-6 gap-2' : 'gap-1.5'">
			<component
				:is="icon ? 'h2' : 'span'"
				v-if="!icon || $slots.heading || heading"
				class="text-2xl font-semibold text-contrast"
				:class="icon ? 'm-0 leading-8' : undefined"
			>
				<slot name="heading">{{ heading }}</slot>
			</component>
			<component
				:is="icon ? 'p' : 'span'"
				v-if="$slots.description || description"
				:class="icon ? 'm-0 text-base leading-6 text-primary' : 'text-secondary'"
			>
				<slot name="description">{{ description }}</slot>
			</component>
		</div>
		<div
			v-if="$slots.actions"
			class="flex gap-2"
			:class="icon ? 'relative mt-6 flex-wrap justify-center' : 'mt-8'"
		>
			<slot name="actions" />
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	DoneIllustration,
	EmptyIllustration,
	EmptyInboxIllustration,
	ErrorIllustration,
	NoConnectionIllustration,
	NoCreditCardIllustration,
	NoDocumentsIllustration,
	NoGPSIllustration,
	NoImagesIllustration,
	NoItemsCartIllustration,
	NoMessagesIllustration,
	NoSearchResultIllustration,
	NoTasksIllustration,
} from '@modrinth/assets'
import type { Component } from 'vue'
import { computed } from 'vue'

const illustrationMap: Record<string, Component> = {
	done: DoneIllustration,
	empty: EmptyIllustration,
	'empty-inbox': EmptyInboxIllustration,
	error: ErrorIllustration,
	'no-connection': NoConnectionIllustration,
	'no-credit-card': NoCreditCardIllustration,
	'no-documents': NoDocumentsIllustration,
	'no-gps': NoGPSIllustration,
	'no-images': NoImagesIllustration,
	'no-items-cart': NoItemsCartIllustration,
	'no-messages': NoMessagesIllustration,
	'no-search-result': NoSearchResultIllustration,
	'no-tasks': NoTasksIllustration,
}

const props = defineProps<{
	type?: keyof typeof illustrationMap
	icon?: Component
	heading?: string
	description?: string
}>()

const illustration = computed(() => (props.type ? illustrationMap[props.type] : undefined))
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
