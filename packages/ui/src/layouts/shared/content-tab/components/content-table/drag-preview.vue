<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import TagItem from '#ui/components/base/TagItem.vue'

import type { ContentCardTableItem } from '../../types'
import ContentItemSummary from '../ContentItemSummary.vue'
import { ROW_GATHER_DURATION } from './use-row-drag-gather'

const props = withDefaults(
	defineProps<{
		row: ContentCardTableItem
		count?: number
	}>(),
	{ count: 1 },
)

const owner = computed(() =>
	props.row.owner ? { ...props.row.owner, link: undefined } : undefined,
)
const source = computed(() =>
	props.row.source ? { ...props.row.source, link: undefined } : undefined,
)
const showGatheredCount = ref(false)
let countTimer: number | undefined

onMounted(() => {
	if (props.count <= 1 || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
		showGatheredCount.value = true
		return
	}
	countTimer = window.setTimeout(() => {
		showGatheredCount.value = true
		countTimer = undefined
	}, ROW_GATHER_DURATION - 150)
})

onBeforeUnmount(() => {
	if (countTimer !== undefined) clearTimeout(countTimer)
})
</script>

<template>
	<div aria-hidden="true" class="relative w-[34rem] max-w-[calc(100vw-2rem)] select-none">
		<div
			v-if="count > 1"
			class="absolute inset-x-3 -bottom-2 top-2 rounded-2xl border border-solid border-surface-5 bg-surface-2 shadow-md motion-safe:transition-opacity motion-safe:delay-100 motion-safe:duration-200 motion-safe:ease-out"
			:class="showGatheredCount ? 'opacity-60' : 'opacity-0'"
		/>
		<div
			v-if="count > 1"
			class="absolute inset-x-1.5 -bottom-1 top-1 rounded-2xl border border-solid border-surface-4 bg-surface-3 shadow-md motion-safe:transition-opacity motion-safe:duration-150 motion-safe:ease-out"
			:class="showGatheredCount ? 'opacity-80' : 'opacity-0'"
		/>
		<div
			class="relative grid grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_auto] items-center gap-x-4 rounded-2xl border border-solid border-surface-5 bg-surface-2 px-3 py-3 opacity-95 shadow-lg"
		>
			<ContentItemSummary
				:project="row.project"
				:version="row.version"
				:show-version="false"
				:owner="owner"
				:source="source"
				:external="row.external"
				:external-file="row.externalFile"
				:override-file="row.overrideFile"
				:client-warning="row.clientWarning"
				:embedded-icon="row.embeddedIcon"
				:dimmed="row.enabled === false"
			/>
			<div
				class="flex min-w-0 flex-col gap-0.5"
				:class="{ 'opacity-50 grayscale': row.enabled === false }"
			>
				<template v-if="row.version">
					<span
						v-if="row.version.version_number"
						class="min-w-0 truncate font-semibold leading-6 text-contrast"
					>
						{{ row.version.version_number }}
					</span>
					<span class="min-w-0 truncate leading-6 text-secondary">
						{{ row.version.file_name }}
					</span>
				</template>
			</div>
			<TagItem
				v-if="count > 1"
				class="shrink-0 border-surface-5 bg-surface-4 tabular-nums text-secondary motion-safe:transition-opacity motion-safe:duration-150"
				:class="showGatheredCount ? 'opacity-100' : 'opacity-0'"
			>
				{{ count }}
			</TagItem>
		</div>
	</div>
</template>
