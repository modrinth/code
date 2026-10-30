<template>
	<div v-if="optionControls.length" :class="groupedOptions ? 'flex flex-col gap-2' : ''">
		<div
			v-for="row in rows"
			:key="row.key"
			class="flex min-w-0 flex-col items-start gap-1"
			:class="{ 'flex-wrap': !groupedOptions }"
		>
			<span v-if="groupedOptions" class="shrink-0 items-center text-xs font-medium text-secondary">
				{{ row.label }}
			</span>
			<div class="flex min-w-0 flex-1 flex-wrap items-center gap-1.5">
				<Button
					v-for="entry in selectedOptionsForRow(row)"
					:key="entry.control.id ?? 'active'"
					size="sm"
					:disabled="pending || disabled || entry.control.disabled"
					:aria-label="`Remove ${entry.control.issueCardDisplay.label}`"
					@click="removeOption(entry)"
				>
					{{ entry.control.issueCardDisplay.label }}
					<XIcon class="size-3" aria-hidden="true" />
				</Button>
				<MultiSelect
					:model-value="groupedOptions ? selectedRowValues(row) : optionDraft"
					:options="optionsForRow(row)"
					:max-tag-rows="3"
					:disabled="pending || disabled"
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
					@update:model-value="groupedOptions ? updateRow(row, $event) : changeOptions($event)"
				>
					<template #input-content>
						<span class="flex items-center gap-1">
							<PlusIcon class="size-4 shrink-0" aria-hidden="true" />
							<span class="sr-only">Add issue option</span>
						</span>
					</template>
				</MultiSelect>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import { Button, MultiSelect, type MultiSelectItem } from '@modrinth/ui'
import { computed, h, ref, watch } from 'vue'

import {
	injectReviewPanels,
	type ReviewIssue,
	type ReviewIssueControl,
} from '~/providers/project-review/review-panels'
import { injectReviewSubmission } from '~/providers/project-review/review-submission-context'

const props = withDefaults(defineProps<{ issue: ReviewIssue; disabled?: boolean }>(), {
	disabled: false,
})
const emit = defineEmits<{ remove: [] }>()
const panels = injectReviewPanels()
const { pending } = injectReviewSubmission()

type ToggleControl = Extract<ReviewIssueControl['control'], { type: 'toggle' }>
interface IssueCardOption extends ReviewIssueControl {
	control: ToggleControl & {
		issueCardDisplay: NonNullable<ToggleControl['issueCardDisplay']>
	}
}
const optionControls = computed(() =>
	props.issue.controls.filter(
		(entry): entry is IssueCardOption =>
			entry.control.type === 'toggle' &&
			entry.control.issueCardDisplay !== undefined &&
			!entry.control.issueCardDisplay.hidden,
	),
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
const groupedOptions = computed(
	() =>
		new Set(optionControls.value.map(({ binding }) => binding.key)).size > 1 ||
		optionControls.value.some(
			(entry) => entry.control.issueCardDisplay.group !== panelLabel(entry),
		),
)
interface ToggleRow {
	key: string
	label: string
	entries: IssueCardOption[]
}

const rows = computed<ToggleRow[]>(() => {
	if (!optionControls.value.length) return []
	if (!groupedOptions.value) return [{ key: 'all', label: '', entries: optionControls.value }]
	const groups = new Map<string, ToggleRow>()
	for (const entry of optionControls.value) {
		const key = entry.control.issueCardDisplay.group
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

function panelLabel(entry: IssueCardOption) {
	return entry.binding.panel.title ?? props.issue.category
}

function optionValue({ control }: IssueCardOption) {
	return control.id === undefined ? `active:${props.issue.id}` : `id:${control.id}`
}

function optionsForRow(row: ToggleRow): MultiSelectItem<string>[] {
	const items: MultiSelectItem<string>[] = []
	const sections = new Map<string, MultiSelectItem<string>[]>()
	for (const entry of row.entries) {
		const { dropdownSection: section, icon, label } = entry.control.issueCardDisplay
		const option = {
			value: optionValue(entry),
			label,
			icon: icon
				? () =>
						h('img', {
							src: icon,
							alt: '',
							class: 'rounded object-contain',
							loading: 'lazy',
						})
				: undefined,
			searchTerms: [entry.control.label, ...(section ? [section] : [])],
			disabled: pending.value || props.disabled || entry.control.disabled,
		}
		if (section) {
			const options = sections.get(section) ?? []
			options.push(option)
			sections.set(section, options)
		} else {
			items.push(option)
		}
	}
	for (const [label, options] of sections) {
		items.push({ type: 'section-header', label, showSelectionActions: false }, ...options)
	}
	return items
}

function removeOption(entry: IssueCardOption) {
	if (pending.value || props.disabled || entry.control.disabled) return
	if (groupedOptions.value && selectedOptionValues.value.length === 1)
		panels.addIssue(props.issue.id)
	panels.write(entry.binding, entry.control, false)
}

function updateRow(row: ToggleRow, values: string[]) {
	if (pending.value || props.disabled) return
	const next = new Set(values)
	if (
		!next.size &&
		!optionControls.value.some(
			(entry) => !row.entries.includes(entry) && panels.selected(entry.binding, entry.control),
		)
	)
		panels.addIssue(props.issue.id)
	const added = row.entries.filter(
		(entry) => !panels.selected(entry.binding, entry.control) && next.has(optionValue(entry)),
	)
	for (const entry of added) {
		panels.write(entry.binding, entry.control, true)
	}
	for (const entry of row.entries) {
		if (panels.selected(entry.binding, entry.control) && !next.has(optionValue(entry)))
			panels.write(entry.binding, entry.control, false)
	}
}

function updateOptions(values: string[]) {
	if (pending.value || props.disabled) return
	const next = new Set(values)
	const added = optionControls.value.filter(
		(entry) => !panels.selected(entry.binding, entry.control) && next.has(optionValue(entry)),
	)
	for (const entry of added) {
		panels.write(entry.binding, entry.control, true)
	}
	for (const entry of optionControls.value) {
		if (panels.selected(entry.binding, entry.control) && !next.has(optionValue(entry)))
			panels.write(entry.binding, entry.control, false)
	}
}

function changeOptions(values: string[]) {
	optionDraft.value = values
	if (values.length) {
		updateOptions(values)
		optionDraft.value = [...selectedOptionValues.value]
	}
}

function openOptions() {
	if (groupedOptions.value) return
	optionDraft.value = [...selectedOptionValues.value]
	optionsOpen.value = true
}

function closeOptions() {
	if (groupedOptions.value) return
	optionsOpen.value = false
	if (pending.value || props.disabled) return
	if (!optionDraft.value.length) {
		if (
			props.issue.controls.some(
				({ binding, control }) =>
					control.type === 'toggle' &&
					(!control.issueCardDisplay || control.issueCardDisplay.hidden) &&
					panels.selected(binding, control),
			)
		) {
			updateOptions([])
		} else {
			emit('remove')
		}
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
