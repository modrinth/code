<template>
	<section class="flex flex-col gap-3" aria-label="File scan patterns">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h3 class="m-0 text-base font-semibold text-contrast">File scan patterns</h3>
			<Button :disabled="disabled" @click="patterns.push({ name: '', pattern: '' })">
				<PlusIcon />
				Add pattern
			</Button>
		</div>
		<Table :columns="columns" :data="rows" row-key="index" :row-below-visible="hasError">
			<template #cell-name="{ row }">
				<Input
					:id="`file-pattern-name-${row.index}`"
					v-model="row.entry.name"
					:disabled="disabled"
					aria-label="Pattern name"
					placeholder="pattern_name"
					autocomplete="off"
					@keydown.enter.prevent
				/>
			</template>
			<template #cell-regex="{ row }">
				<Input
					:id="`file-pattern-regex-${row.index}`"
					v-model="row.entry.pattern"
					:disabled="disabled"
					aria-label="Regex pattern"
					placeholder="(?i)forbidden text"
					input-class="font-mono"
					autocomplete="off"
					:spellcheck="false"
					@keydown.enter.prevent
				/>
			</template>
			<template #header-actions>
				<span class="sr-only">Remove</span>
			</template>
			<template #cell-actions="{ row }">
				<IconButton
					:disabled="disabled"
					:circular="false"
					:label="`Remove pattern ${row.index + 1}`"
					@click="patterns.splice(row.index, 1)"
				>
					<TrashIcon />
				</IconButton>
			</template>
			<template #row-below="{ row }">
				<p role="alert" class="m-0 px-4 pb-2 text-sm text-red">{{ errors[row.index] }}</p>
			</template>
			<template #empty-state><span /></template>
		</Table>
	</section>
</template>

<script setup lang="ts">
import { PlusIcon, TrashIcon } from '@modrinth/assets'
import { Button, IconButton, Input, Table, type TableColumn } from '@modrinth/ui'
import { computed } from 'vue'

export interface FileScanPattern {
	name: string
	pattern: string
}

const patterns = defineModel<FileScanPattern[]>({ required: true })
const props = defineProps<{ disabled?: boolean; errors: string[] }>()
const rows = computed(() => patterns.value.map((entry, index) => ({ entry, index })))
const columns: TableColumn<'name' | 'regex' | 'actions'>[] = [
	{ key: 'name', label: 'Name', width: '35%', cellClass: 'pr-2 py-2' },
	{ key: 'regex', label: 'Regex', cellClass: 'pr-2 py-2' },
	{ key: 'actions', width: '3.5rem', align: 'right' },
]

function hasError(row: { index: number }) {
	return Boolean(props.errors[row.index])
}
</script>
