<template>
	<section ref="section" class="flex min-w-0 flex-col gap-1" :aria-labelledby="headingId">
		<div class="flex min-h-[38px] flex-wrap items-center justify-between gap-2">
			<div class="flex items-center gap-1.5">
				<h3 :id="headingId" class="m-0 text-sm font-semibold text-primary">Questions</h3>
				<button
					v-tooltip="{
						text: decisionQuestionsHelp,
						class: 'max-w-[min(22rem,calc(100vw-2rem))] whitespace-normal',
					}"
					type="button"
					aria-label="About decision questions"
					:aria-describedby="helpId"
					class="inline-flex cursor-help rounded-full border-0 bg-transparent p-0 text-secondary hover:text-contrast focus-visible:text-contrast"
				>
					<InfoIcon class="size-4" aria-hidden="true" />
				</button>
				<span :id="helpId" hidden>{{ decisionQuestionsHelp }}</span>
			</div>
			<Button
				type="quiet"
				size="sm"
				aria-label="Add question"
				:disabled="disabled"
				@click="addQuestion"
			>
				<PlusIcon />
				Add
			</Button>
		</div>
		<p v-if="!entries.length" class="m-0 text-sm text-secondary">No questions.</p>
		<ul v-else class="m-0 flex list-none flex-col gap-2 p-0">
			<li
				v-for="(entry, index) in entries"
				:key="rowId(entry)"
				class="overflow-hidden rounded-xl border border-solid border-surface-4 bg-surface-3"
			>
				<Accordion
					:open-by-default="expandedQuestions.has(entry)"
					button-class="flex w-full cursor-pointer items-center gap-2 border-0 bg-surface-3 px-3 py-2 text-left text-sm font-medium text-contrast transition-[filter] hover:brightness-125 focus-visible:-outline-offset-2 focus-visible:brightness-125 motion-reduce:transition-none"
					content-class="flex min-w-0 flex-col gap-3 px-3 pb-3 pt-1"
					@on-open="expandedQuestions.add(entry)"
					@on-close="expandedQuestions.delete(entry)"
				>
					<template #button="{ open }">
						<span class="min-w-0 flex-1 truncate">{{ entry.key.trim() || 'New question' }}</span>
						<span v-if="errors[index]" class="shrink-0 text-red">Error</span>
						<ChevronDownIcon
							class="size-4 shrink-0 text-secondary transition-transform duration-300 motion-reduce:transition-none"
							:class="{ 'rotate-180': open }"
							aria-hidden="true"
						/>
					</template>
					<div class="flex min-w-0 flex-col gap-1">
						<label :for="`${rowId(entry)}-key`" class="text-sm font-medium text-secondary"
							>Key</label
						>
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
					<div class="flex min-w-0 flex-col gap-1">
						<label :for="`${rowId(entry)}-question`" class="text-sm font-medium text-secondary">
							Question
						</label>
						<Textarea
							:id="`${rowId(entry)}-question`"
							:model-value="entry.question"
							:disabled="disabled"
							:rows="5"
							placeholder="Does this text endorse cheating in any way?"
							autocomplete="off"
							@update:model-value="entry.question = $event ?? ''"
						/>
					</div>
					<div class="flex min-w-0 flex-col gap-1">
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
					<p v-if="errors[index]" role="alert" class="m-0 text-sm text-red">
						{{ errors[index] }}
					</p>
					<div class="flex min-w-0 items-center gap-1">
						<p class="m-0 min-w-0 flex-1 break-words text-xs text-secondary">
							Answers appear by key in <code class="text-link">ai.decisions</code> as a confidence
							value from 0 to 1.
						</p>
						<IconButton
							class="shrink-0 enabled:hover:text-red"
							type="quiet"
							size="sm"
							:disabled="disabled"
							:label="`Remove question ${index + 1}`"
							@click="removeQuestion(index)"
						>
							<TrashIcon aria-hidden="true" />
						</IconButton>
					</div>
				</Accordion>
			</li>
		</ul>
	</section>
</template>

<script setup lang="ts">
import type { ProjectFlagContextField, ProjectFlagDecisionQuestion } from '@modrinth/api-client'
import { ChevronDownIcon, InfoIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import {
	Accordion,
	Button,
	IconButton,
	Input,
	MultiSelect,
	type MultiSelectOption,
	Textarea,
} from '@modrinth/ui'
import { nextTick, reactive, useId, useTemplateRef } from 'vue'

import { useUndoableDraftRemoval } from './use-undoable-draft-removal'

export interface DecisionQuestionEntry extends ProjectFlagDecisionQuestion {
	key: string
}

const decisionQuestionsHelp =
	'Each question uses only its selected project context. Each uncached question makes an independent AI call, including in the tester.'

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
const props = defineProps<{ disabled?: boolean; errors: string[] }>()
const removeQuestion = useUndoableDraftRemoval(
	entries,
	() => props.disabled,
	({ key }) => {
		const trimmedKey = key.trim()
		return { title: trimmedKey ? `Question “${trimmedKey}” removed` : 'Question removed' }
	},
)

const id = useId()
const headingId = `${id}-heading`
const helpId = `${id}-help`
const section = useTemplateRef<HTMLElement>('section')
const expandedQuestions = reactive(new WeakSet<DecisionQuestionEntry>())
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
	const entry = entries.value[entries.value.length - 1]
	expandedQuestions.add(entry)
	const keyId = `${rowId(entry)}-key`
	await nextTick()
	section.value?.querySelector<HTMLInputElement>(`[id="${keyId}"]`)?.focus()
}
</script>
