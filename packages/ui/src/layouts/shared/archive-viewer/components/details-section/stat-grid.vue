<template>
	<dl
		class="m-0 gap-2 text-sm"
		:class="inline ? 'flex flex-wrap' : 'grid grid-cols-[repeat(auto-fill,minmax(8.5rem,1fr))]'"
	>
		<div
			v-for="stat in stats"
			:key="stat.id"
			class="flex min-w-0 rounded-lg border border-solid border-surface-4 bg-surface-2 px-2"
			:class="
				inline || grid ? 'max-w-full flex items-center gap-1.5 py-1' : 'flex-col gap-1 py-1.5'
			"
		>
			<dt class="text-xs font-medium text-secondary flex-grow">{{ `${stat.label}: ` }}</dt>
			<dd class="m-0 min-w-0">
				<SpinnerIcon
					v-if="stat.loading"
					class="size-4 animate-spin text-secondary"
					aria-hidden="true"
				/>
				<HashValue v-else-if="stat.hash && stat.value" :hash="stat.value" :display="stat.display" />
				<span
					v-else
					v-tooltip="stat.tooltip"
					class="block truncate"
					:class="[
						stat.value == null ? 'text-secondary' : 'text-primary',
						{ 'cursor-help': stat.tooltip },
					]"
				>
					{{ stat.value ?? '-' }}
				</span>
			</dd>
		</div>
	</dl>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'

import type { ArchiveDetailRow } from '../../types'
import HashValue from '../HashValue.vue'

/**
 * Short values (sizes, checksums) laid out side by side, as many to a row as fit, each with its
 * label above it, or beside it when `inline`.
 */
defineProps<{
	stats: ArchiveDetailRow[]
	inline?: boolean
	grid?: boolean
}>()
</script>
