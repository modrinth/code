<template>
	<SpinnerIcon v-if="loading" class="size-4 animate-spin text-secondary" aria-hidden="true" />
	<span v-else class="text-nowrap text-sm text-secondary">{{ value ?? '-' }}</span>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { ref, watch } from 'vue'

import type { FileEntryDetail, FileItem } from '../types'

/** Resolves a custom {@link FileEntryDetail}, which may load lazily (e.g. hashing the file). */
const props = defineProps<{
	detail: FileEntryDetail
	entry: FileItem
}>()

const value = ref<string | null>(null)
const loading = ref(false)

watch(
	() => [props.detail, props.entry] as const,
	async ([detail, entry], _, onCleanup) => {
		let cancelled = false
		onCleanup(() => (cancelled = true))

		loading.value = true
		const result = await Promise.resolve(detail.value(entry)).catch(() => null)
		if (cancelled) return
		value.value = result
		loading.value = false
	},
	{ immediate: true },
)
</script>
