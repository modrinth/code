<template>
	<div class="flex min-w-0 flex-col gap-2">
		<p class="m-0 break-words text-sm font-medium text-contrast">
			{{ flag.name || flag.key }}
		</p>
		<p v-if="flag.reason" class="m-0 whitespace-pre-wrap break-words">{{ flag.reason }}</p>
		<span
			v-if="flag.location?.review_panel_key"
			class="inline-flex w-fit max-w-full items-center gap-1 rounded-full bg-surface-4 px-2 py-1 text-xs"
		>
			<PanelTopIcon class="size-3.5 shrink-0" aria-hidden="true" />
			<span class="min-w-0 break-words">{{ flag.location.review_panel_key }}</span>
		</span>
		<Accordion
			button-class="m-0 w-full cursor-pointer border-none bg-transparent p-0 text-left text-sm font-medium [&>div>svg]:ml-0 [&>div>svg]:size-4"
			content-class="pt-1"
		>
			<template #title>Flag data</template>
			<pre
				class="m-0 max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-xl bg-surface-2"
				>{{ JSON.stringify(flag, null, 2) }}</pre
			>
		</Accordion>
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlag } from '@modrinth/api-client'
import { PanelTopIcon } from '@modrinth/assets'
import { Accordion } from '@modrinth/ui'

defineProps<{ flag: ProjectFlag }>()
</script>
