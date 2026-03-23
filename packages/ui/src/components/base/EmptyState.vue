<template>
	<div class="mx-auto flex flex-col items-center p-6 text-center">
		<component :is="illustration" v-if="illustration" class="h-[200px] w-auto" />
		<div class="flex flex-col items-center gap-1.5">
			<span class="text-2xl font-semibold text-contrast">
				<slot name="heading">{{ heading }}</slot>
			</span>
			<span v-if="$slots.description || description" class="text-secondary">
				<slot name="description">{{ description }}</slot>
			</span>
		</div>
		<div v-if="$slots.actions" class="mt-8 flex gap-2">
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
	heading?: string
	description?: string
}>()

const illustration = computed(() => (props.type ? illustrationMap[props.type] : undefined))
</script>
