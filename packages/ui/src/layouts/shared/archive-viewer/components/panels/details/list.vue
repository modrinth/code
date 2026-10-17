<template>
	<dl class="m-0 grid grid-cols-[minmax(6rem,max-content)_minmax(0,1fr)] gap-x-4 gap-y-2 text-sm">
		<template v-for="row in rows" :key="row.id">
			<dt class="text-secondary">{{ row.label }}</dt>
			<dd class="m-0 min-w-0 break-words text-primary">
				<SpinnerIcon v-if="row.loading" class="size-4 animate-spin text-secondary" aria-hidden="true" />
				<HashValue
					v-else-if="row.hash && row.value"
					:hash="row.value"
					:display="row.display"
				/>
				<CopyCode
					v-else-if="row.copyable && row.value"
					class="max-w-full [&>span]:truncate"
					:text="row.value"
				/>
				<a
					v-else-if="row.href && row.value"
					:href="row.href"
					target="_blank"
					rel="noopener noreferrer"
					class="break-all text-link hover:underline"
				>
					{{ row.value }}
				</a>
				<span v-else :class="{ 'text-secondary': row.value == null }">{{ row.value ?? '-' }}</span>
			</dd>
		</template>
	</dl>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'

import CopyCode from '#ui/components/base/CopyCode.vue'

import type { ArchiveDetailRow } from '../../../types'
import HashValue from '../../HashValue.vue'

defineProps<{
	rows: ArchiveDetailRow[]
}>()
</script>
