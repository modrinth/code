<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { computed } from 'vue'

import { severityPresentation, traceName } from './presentation'

const props = defineProps<{
	issueType: string
	severity: Labrinth.TechReview.Internal.DelphiSeverity
	colored?: boolean
}>()

const presentation = computed(() => severityPresentation[props.severity])
const severityLabel = computed(
	() => props.severity.charAt(0).toUpperCase() + props.severity.slice(1),
)
</script>

<template>
	<span class="inline-flex min-w-0 items-center gap-1.5">
		<span
			v-tooltip="severityLabel"
			:aria-label="severityLabel"
			class="mx-0.5 inline-flex shrink-0 items-center justify-center rounded-md"
			:class="presentation.class"
		>
			<component :is="presentation.icon" class="size-4" aria-hidden="true" />
		</span>
		<span
			class="min-w-0 break-words text-sm font-medium"
			:class="colored ? presentation.textClass : 'text-contrast'"
			>{{ traceName(issueType) }}</span
		>
	</span>
</template>
