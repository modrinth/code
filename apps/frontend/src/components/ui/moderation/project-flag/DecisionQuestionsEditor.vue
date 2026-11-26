<template>
	<section class="flex flex-col gap-3" aria-label="Decision questions">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h3 class="m-0 text-base font-semibold text-contrast">Decision questions</h3>
			<Button :disabled="disabled" @click="addQuestion">
				<PlusIcon />
				Add question
			</Button>
		</div>
		<p class="m-0 text-sm text-secondary">
			Each question uses only its selected project context. Each uncached question makes an
			independent AI call, including in the tester. Answers remain available by key in ai.decisions.
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
				<fieldset class="m-0 mt-2 flex flex-col gap-2 border-0 p-0" :disabled="disabled">
					<legend class="mb-2 text-sm font-semibold">Include context</legend>
					<Checkbox
						v-for="field in contextFields"
						:key="field.value"
						:label="field.label"
						:disabled="disabled"
						:model-value="row.entry.include_context.includes(field.value)"
						@update:model-value="setContext(row.entry, field.value, $event)"
					/>
				</fieldset>
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
import type { ProjectFlagContextField, ProjectFlagDecisionQuestion } from '@modrinth/api-client'
import { PlusIcon, TrashIcon } from '@modrinth/assets'
import { Button, Checkbox, IconButton, Input, Table, type TableColumn } from '@modrinth/ui'
import { computed } from 'vue'

export interface DecisionQuestionEntry extends ProjectFlagDecisionQuestion {
	key: string
}

const contextFields: { value: ProjectFlagContextField; label: string }[] = [
	{ value: 'title', label: 'Title' },
	{ value: 'summary', label: 'Summary' },
	{ value: 'description', label: 'Description' },
	{ value: 'links', label: 'Links' },
	{ value: 'slug', label: 'Slug' },
	{ value: 'license', label: 'License (ID, name, URL)' },
	{ value: 'categories', label: 'Categories' },
	{ value: 'additional_categories', label: 'Additional categories' },
	{ value: 'gallery_titles', label: 'Gallery titles' },
	{ value: 'gallery_descriptions', label: 'Gallery descriptions' },
	{ value: 'version_names', label: 'Version names' },
	{ value: 'version_numbers', label: 'Version numbers' },
	{ value: 'version_changelogs', label: 'Version changelogs' },
	{ value: 'filenames', label: 'Version filenames' },
	{ value: 'server_addresses', label: 'Server listing addresses (Java/Bedrock)' },
]

function addQuestion() {
	entries.value.push({
		key: '',
		question: '',
		include_context: ['title', 'summary', 'description'],
	})
}

function setContext(
	entry: DecisionQuestionEntry,
	field: ProjectFlagContextField,
	selected: boolean,
) {
	entry.include_context = selected
		? [...entry.include_context, field]
		: entry.include_context.filter((value) => value !== field)
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
