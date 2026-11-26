<template>
	<section ref="section" class="flex min-w-0 flex-col gap-3" :aria-labelledby="headingId">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<div class="flex items-center gap-1.5">
				<h3 :id="headingId" class="m-0 text-base font-semibold text-contrast">
					Decision questions
				</h3>
				<button
					v-tooltip="decisionQuestionsHelp"
					type="button"
					aria-label="About decision questions"
					:aria-describedby="helpId"
					class="inline-flex cursor-help rounded-full border-0 bg-transparent p-0 text-secondary hover:text-contrast focus-visible:text-contrast"
				>
					<InfoIcon class="size-4" aria-hidden="true" />
				</button>
				<span :id="helpId" hidden>{{ decisionQuestionsHelp }}</span>
			</div>
			<Button type="quiet" :disabled="disabled" @click="addQuestion">
				<PlusIcon />
				Add question
			</Button>
		</div>
		<p v-if="entries.length === 0" class="m-0 text-sm text-secondary">No decision questions.</p>
		<ul v-else class="m-0 flex list-none flex-col gap-2 p-0">
			<li
				v-for="(entry, index) in entries"
				:key="rowId(entry)"
				class="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-start gap-2 rounded-xl border border-solid border-surface-4 p-3"
			>
				<div class="flex min-w-0 flex-col gap-1">
					<label :for="`${rowId(entry)}-key`" class="text-sm font-medium text-secondary">Key</label>
					<Input
						:id="`${rowId(entry)}-key`"
						v-model="entry.key"
						:disabled="disabled"
						placeholder="endorses_cheating"
						autocomplete="off"
						:spellcheck="false"
						@keydown.enter.prevent
					/>
				</div>
				<IconButton
					class="mt-6 enabled:hover:text-red"
					type="quiet"
					:disabled="disabled"
					:circular="false"
					:label="`Remove question ${index + 1}`"
					@click="entries.splice(index, 1)"
				>
					<TrashIcon />
				</IconButton>
				<div class="col-span-full flex min-w-0 flex-col gap-1">
					<label :for="`${rowId(entry)}-question`" class="text-sm font-medium text-secondary">
						Question
					</label>
					<Input
						:id="`${rowId(entry)}-question`"
						v-model="entry.question"
						:disabled="disabled"
						placeholder="Does this text endorse cheating in any way?"
						autocomplete="off"
						@keydown.enter.prevent
					/>
				</div>
				<div class="col-span-full flex min-w-0 flex-col gap-1">
					<span :id="`${rowId(entry)}-context`" class="text-sm font-medium text-secondary">
						Include context
					</span>
					<MultiSelect
						v-model="entry.include_context"
						:options="contextFields"
						:disabled="disabled"
						:aria-labelledby="`${rowId(entry)}-context`"
						role="group"
						placeholder="Select context fields"
						searchable
						search-placeholder="Search context fields"
					/>
				</div>
				<p v-if="errors[index]" role="alert" class="col-span-full m-0 text-sm text-red">
					{{ errors[index] }}
				</p>
			</li>
		</ul>
	</section>
</template>

<script setup lang="ts">
import type { ProjectFlagContextField, ProjectFlagDecisionQuestion } from '@modrinth/api-client'
import { InfoIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import { Button, IconButton, Input, MultiSelect, type MultiSelectOption } from '@modrinth/ui'
import { nextTick, useId, useTemplateRef } from 'vue'

export interface DecisionQuestionEntry extends ProjectFlagDecisionQuestion {
	key: string
}

const decisionQuestionsHelp =
	'Each question uses only its selected project context. Each uncached question makes an independent AI call, including in the tester. Answers remain available by key in "ai.decisions".'

const contextFields: MultiSelectOption<ProjectFlagContextField>[] = [
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

const entries = defineModel<DecisionQuestionEntry[]>({ required: true })
defineProps<{ disabled?: boolean; errors: string[] }>()

const id = useId()
const headingId = `${id}-heading`
const helpId = `${id}-help`
const section = useTemplateRef<HTMLElement>('section')
const rowIds = new WeakMap<DecisionQuestionEntry, string>()
let nextRowId = 0

function rowId(entry: DecisionQuestionEntry) {
	let row = rowIds.get(entry)
	if (!row) {
		row = `${id}-question-${nextRowId++}`
		rowIds.set(entry, row)
	}
	return row
}

async function addQuestion() {
	entries.value.push({
		key: '',
		question: '',
		include_context: ['title', 'summary', 'description'],
	})
	const keyId = `${rowId(entries.value[entries.value.length - 1])}-key`
	await nextTick()
	section.value?.querySelector<HTMLInputElement>(`[id="${keyId}"]`)?.focus()
}
</script>
