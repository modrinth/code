<template>
	<StatGrid v-if="stats.length" :stats="stats" grid />
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import { useVIntl } from '#ui/composables/i18n'
import type { HashType } from '#ui/helpers/toolbox/file'

import { hashTypeMessages } from '../../messages'
import type { ArchiveDetailRow } from '../../types'
import StatGrid from './stat-grid.vue'

/**
 * The item's hashes of the given types, each computed only once it's listed here and kept for as
 * long as the item stays the same.
 */
const props = defineProps<{
	/** Computes the item's hash of a type; a new function means a different item. */
	resolve: (type: HashType) => Promise<string>
	types: HashType[]
}>()

const { formatMessage } = useVIntl()

interface HashResult {
	value: string | null
	loading: boolean
}

const results = ref<Partial<Record<HashType, HashResult>>>({})

watch(
	[() => props.resolve, () => props.types],
	([resolve, types], previous) => {
		if (previous && previous[0] !== resolve) results.value = {}
		for (const type of types) {
			if (results.value[type]) continue
			results.value = { ...results.value, [type]: { value: null, loading: true } }
			resolve(type)
				.then((value) => ({ value, loading: false }))
				.catch(() => ({ value: null, loading: false }))
				.then((result) => {
					if (props.resolve === resolve) results.value = { ...results.value, [type]: result }
				})
		}
	},
	{ immediate: true },
)

const stats = computed<ArchiveDetailRow[]>(() =>
	props.types.map((type) => ({
		id: type,
		label: formatMessage(hashTypeMessages[type]),
		value: results.value[type]?.value ?? null,
		loading: results.value[type]?.loading ?? true,
		hash: true,
	})),
)
</script>
