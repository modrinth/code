<template>
	<section class="flex flex-col gap-3" aria-label="Decision questions">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h3 class="m-0 text-base font-semibold text-contrast">Decision questions</h3>
			<Button :disabled="disabled" @click="entries.push({ key: '', question: '' })">
				<PlusIcon />
				Add question
			</Button>
		</div>
		<p class="m-0 text-sm text-secondary">
			Key/question pairs are stored with the rule and passed to the test API. The tester does not
			evaluate the decision model.
		</p>
		<Table :columns="columns" :data="rows" row-key="index" :row-below-visible="hasError">
			<template #cell-key="{ row }">
				<Input
					:id="`decision-questions-key-${row.index}`"
					v-model="row.entry.key"
					:disabled="disabled"
					aria-label="Decision question key"
					placeholder="endorses_cheating"
					autocomplete="off"
					:spellcheck="false"
					@keydown.enter.prevent
				/>
			</template>
			<template #cell-question="{ row }">
				<Input
					:id="`decision-questions-question-${row.index}`"
					v-model="row.entry.question"
					:disabled="disabled"
					aria-label="Decision question"
					placeholder="Does this text endorse cheating in any way?"
					autocomplete="off"
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
					:label="`Remove question ${row.index + 1}`"
					@click="entries.splice(row.index, 1)"
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

export interface DecisionQuestionEntry {
	key: string
	question: string
}

const entries = defineModel<DecisionQuestionEntry[]>({ required: true })
const props = defineProps<{ disabled?: boolean; errors: string[] }>()
const rows = computed(() => entries.value.map((entry, index) => ({ entry, index })))
const columns: TableColumn<'key' | 'question' | 'actions'>[] = [
	{ key: 'key', label: 'Key', width: '35%', cellClass: 'pr-2 py-2' },
	{ key: 'question', label: 'Question', cellClass: 'pr-2 py-2' },
	{ key: 'actions', width: '3.5rem', align: 'right' },
]

function hasError(row: { index: number }) {
	return Boolean(props.errors[row.index])
}
</script>
