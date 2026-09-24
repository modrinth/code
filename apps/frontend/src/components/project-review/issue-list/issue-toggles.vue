<template>
	<div v-if="optionControls.length > 1" :class="multiplePanels ? 'flex flex-col gap-2' : ''">
		<div
			v-for="row in rows"
			:key="row.key"
			class="flex min-w-0 flex-col items-start gap-1"
			:class="{ 'flex-wrap': !multiplePanels }"
		>
			<span v-if="multiplePanels" class="shrink-0 items-center text-xs font-medium text-secondary">
				{{ row.label }}
			</span>
			<div class="flex min-w-0 flex-1 flex-wrap items-center gap-1.5">
				<Button
					v-for="entry in selectedOptionsForRow(row)"
					:key="
						entry.control.type === 'toggle' ? (entry.control.id ?? 'active') : entry.control.key
					"
					size="sm"
					:disabled="pending || entry.control.disabled"
					:aria-label="formatMessage(messages.removeOption, { option: optionLabel(entry) })"
					@click="removeOption(entry)"
				>
					{{ multiplePanels ? rowOptionLabel(entry) : optionLabel(entry) }}
					<XIcon class="size-3" aria-hidden="true" />
				</Button>
				<MultiSelect
					:model-value="multiplePanels ? selectedRowValues(row) : optionDraft"
					:options="optionsForRow(row)"
					:disabled="pending"
					:dropdown-width="280"
					lock-dropdown-horizontal-position
					dropdown-align="left"
					:max-height="440"
					:clearable="false"
					:show-chevron="false"
					trigger-class="!size-8 !px-0"
					fit-content
					:searchable="row.entries.length >= 6"
					trigger-type="base"
					trigger-size="sm"
					checkbox-position="right"
					@open="openOptions"
					@close="closeOptions"
					@update:model-value="multiplePanels ? updateRow(row, $event) : changeOptions($event)"
				>
					<template #input-content>
						<span class="flex items-center gap-1">
							<PlusIcon class="size-4 shrink-0" aria-hidden="true" />
							<span class="sr-only">{{ formatMessage(messages.addOption) }}</span>
						</span>
					</template>
				</MultiSelect>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import { Button, defineMessages, MultiSelect, type MultiSelectItem, useVIntl } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { injectReviewMessages } from '~/providers/project-review/review-messages'
import {
	injectReviewPanels,
	type ReviewIssue,
	type ReviewIssueControl,
} from '~/providers/project-review/review-panels'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

const props = defineProps<{ issue: ReviewIssue }>()
const panels = injectReviewPanels()
const reviewMessages = injectReviewMessages()
const { pending } = injectReviewSubmission()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	addOption: {
		id: 'project-review.issues.add-option',
		defaultMessage: 'Add issue option',
	},
	removeOption: {
		id: 'project-review.issues.remove-option',
		defaultMessage: 'Remove {option}',
	},
	fieldOption: {
		id: 'project-review.issues.field-option',
		defaultMessage: '{field}: {option}',
	},
})
const optionControls = computed(() =>
	props.issue.controls.filter(({ control }) => control.type === 'toggle'),
)
const selectedOptionValues = computed(() =>
	optionControls.value
		.filter(({ binding, control }) => panels.selected(binding, control))
		.map(optionValue),
)
const optionDraft = ref([...selectedOptionValues.value])
const optionsOpen = ref(false)
const selectedOptions = computed(() => {
	const draft = new Set(optionDraft.value)
	return optionControls.value.filter((entry) =>
		optionsOpen.value
			? draft.has(optionValue(entry))
			: panels.selected(entry.binding, entry.control),
	)
})
const multiplePanels = computed(
	() => new Set(optionControls.value.map(({ binding }) => binding.key)).size > 1,
)
interface ToggleRow {
	key: string
	label: string
	entries: ReviewIssueControl[]
}

const rows = computed<ToggleRow[]>(() => {
	if (!multiplePanels.value) return [{ key: 'all', label: '', entries: optionControls.value }]
	const groups = new Map<string, ToggleRow>()
	for (const entry of optionControls.value) {
		const key =
			entry.control.type === 'toggle'
				? (entry.control.issueListGroup ?? entry.control.label)
				: entry.control.label
		const row = groups.get(key) ?? { key, label: key, entries: [] }
		row.entries.push(entry)
		groups.set(key, row)
	}
	return [...groups.values()]
})

function selectedOptionsForRow(row: ToggleRow) {
	return selectedOptions.value.filter((entry) => row.entries.includes(entry))
}

function selectedRowValues(row: ToggleRow) {
	return row.entries
		.filter(({ binding, control }) => panels.selected(binding, control))
		.map(optionValue)
}

function optionLabel(entry: ReviewIssueControl) {
	if (entry.control.type === 'toggle' && entry.control.issueListLabel)
		return entry.control.issueListLabel
	if (!multiplePanels.value) return entry.control.label
	return formatMessage(messages.fieldOption, {
		field: entry.binding.panel.title,
		option: entry.control.label,
	})
}

function rowOptionLabel(entry: ReviewIssueControl) {
	return entry.control.type === 'toggle'
		? (entry.control.issueListLabel ?? entry.binding.panel.title)
		: entry.binding.panel.title
}

function optionValue({ control }: ReviewIssueControl) {
	return control.type === 'toggle'
		? control.id === undefined
			? `active:${props.issue.id}`
			: `id:${control.id}`
		: ''
}

const optionItems = computed<MultiSelectItem<string>[]>(() => {
	return optionControls.value.map((entry) => ({
		value: optionValue(entry),
		label: optionLabel(entry),
		searchTerms: multiplePanels.value ? [entry.control.label] : [],
		disabled: pending.value || entry.control.disabled,
	}))
})

function optionsForRow(row: ToggleRow): MultiSelectItem<string>[] {
	if (!multiplePanels.value) return optionItems.value
	return row.entries.map((entry) => ({
		value: optionValue(entry),
		label: rowOptionLabel(entry),
		disabled: pending.value || entry.control.disabled,
	}))
}

function removeOption(entry: ReviewIssueControl) {
	if (pending.value || entry.control.disabled) return
	if (multiplePanels.value && selectedOptionValues.value.length === 1)
		panels.addIssue(props.issue.id)
	panels.write(entry.binding, entry.control, false)
}

function updateRow(row: ToggleRow, values: string[]) {
	if (pending.value) return
	const next = new Set(values)
	if (
		!next.size &&
		!optionControls.value.some(
			(entry) => !row.entries.includes(entry) && panels.selected(entry.binding, entry.control),
		)
	)
		panels.addIssue(props.issue.id)
	for (const entry of row.entries) {
		if (!panels.selected(entry.binding, entry.control) && next.has(optionValue(entry)))
			panels.write(entry.binding, entry.control, true)
	}
	for (const entry of row.entries) {
		if (panels.selected(entry.binding, entry.control) && !next.has(optionValue(entry)))
			panels.write(entry.binding, entry.control, false)
	}
}

function updateOptions(values: string[]) {
	if (pending.value) return
	const next = new Set(values)
	for (const entry of optionControls.value) {
		if (!panels.selected(entry.binding, entry.control) && next.has(optionValue(entry)))
			panels.write(entry.binding, entry.control, true)
	}
	for (const entry of optionControls.value) {
		if (panels.selected(entry.binding, entry.control) && !next.has(optionValue(entry)))
			panels.write(entry.binding, entry.control, false)
	}
}

function changeOptions(values: string[]) {
	optionDraft.value = values
	if (values.length) updateOptions(values)
}

function openOptions() {
	if (multiplePanels.value) return
	optionDraft.value = [...selectedOptionValues.value]
	optionsOpen.value = true
}

function closeOptions() {
	if (multiplePanels.value) return
	optionsOpen.value = false
	if (pending.value) return
	if (!optionDraft.value.length) {
		panels.removeIssue(props.issue.id)
		reviewMessages.resetIssueMessage(props.issue.id)
		return
	}
	const selected = new Set(selectedOptionValues.value)
	if (
		optionDraft.value.length !== selected.size ||
		optionDraft.value.some((value) => !selected.has(value))
	) {
		updateOptions(optionDraft.value)
	}
}

watch(selectedOptionValues, (value) => {
	if (!optionsOpen.value) optionDraft.value = [...value]
})
</script>
