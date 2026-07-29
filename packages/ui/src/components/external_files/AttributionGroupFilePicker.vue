<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { computed } from 'vue'

import { Checkbox } from '#ui/components'

const props = defineProps<{
	files: Labrinth.Attribution.Internal.AttributionFile[]
	disabled?: boolean
}>()

const selectedSha1s = defineModel<Set<string>>('selectedSha1s', { required: true })

const allSelected = computed(
	() => props.files.length > 0 && props.files.every((file) => selectedSha1s.value.has(file.sha1)),
)

const someSelected = computed(
	() => props.files.some((file) => selectedSha1s.value.has(file.sha1)) && !allSelected.value,
)

function displayName(file: Labrinth.Attribution.Internal.AttributionFile) {
	return file.name.split('/').pop() ?? file.name
}

function setAllSelected(selected: boolean) {
	const next = new Set(selectedSha1s.value)
	for (const file of props.files) {
		if (selected) {
			next.add(file.sha1)
		} else {
			next.delete(file.sha1)
		}
	}
	selectedSha1s.value = next
}

function toggleFile(sha1: string, selected: boolean) {
	const next = new Set(selectedSha1s.value)
	if (selected) {
		next.add(sha1)
	} else {
		next.delete(sha1)
	}
	selectedSha1s.value = next
}
</script>

<template>
	<div v-if="files.length > 1" class="flex flex-col gap-2">
		<Checkbox
			:model-value="allSelected"
			:indeterminate="someSelected"
			label="Select files to add"
			:disabled="disabled"
			@update:model-value="setAllSelected"
		/>
		<div class="flex flex-col [&>*:nth-child(even)]:bg-surface-3">
			<Checkbox
				v-for="file in files"
				:key="file.sha1"
				:model-value="selectedSha1s.has(file.sha1)"
				:label="displayName(file)"
				:disabled="disabled"
				class="w-full px-4 py-2 hover:bg-surface-4 text-primary"
				@update:model-value="(selected) => toggleFile(file.sha1, selected)"
				@click.stop
			/>
		</div>
	</div>
</template>
