<script setup lang="ts">
import type { Component } from 'vue'

import ToggleCard from './ToggleCard.vue'

defineProps<{
	title: string
	icon?: Component
	description?: string
	disabled?: boolean
	toggleDisabled?: boolean
}>()

const enabled = defineModel<boolean>({ required: true })
</script>

<template>
	<ToggleCard v-model="enabled" :disabled="disabled" :toggle-disabled="toggleDisabled">
		<h3 class="mb-1 mt-0 flex items-center gap-2 text-lg font-semibold text-contrast">
			<component :is="icon" v-if="icon" class="size-5 shrink-0 text-primary" aria-hidden="true" />
			{{ title }}
		</h3>
		<div v-if="description || $slots.default" class="flex flex-col gap-2 leading-normal [&>p]:m-0">
			<p v-if="description">{{ description }}</p>
			<slot />
		</div>
		<template v-if="$slots.expanded" #expanded>
			<div class="flex flex-col gap-4">
				<slot name="expanded" />
			</div>
		</template>
		<template v-if="$slots.footer" #footer>
			<slot name="footer" />
		</template>
	</ToggleCard>
</template>
