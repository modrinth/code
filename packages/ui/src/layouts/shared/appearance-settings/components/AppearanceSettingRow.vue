<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
	defineProps<{
		controlId: string
		title: string
		description: string
		headingLevel?: 2 | 3
	}>(),
	{
		headingLevel: 2,
	},
)

const headingTag = computed(() => `h${props.headingLevel}`)
</script>

<template>
	<div class="flex items-center justify-between gap-4">
		<div class="min-w-0">
			<component
				:is="headingTag"
				:id="`${controlId}-label`"
				class="m-0 text-lg font-semibold text-contrast"
			>
				{{ title }}
			</component>
			<p class="m-0 mt-1 text-secondary">
				{{ description }}
			</p>
		</div>
		<div class="shrink-0">
			<slot :labelled-by="`${controlId}-label`" />
		</div>
	</div>
</template>
