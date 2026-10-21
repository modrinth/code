<template>
	<MultiSelect
		ref="picker"
		:model-value="draft"
		:options="options"
		:max-tag-rows="3"
		:disabled="pending"
		:searchable="true"
		search-placeholder="Search issues…"
		no-options-message="No matching issues."
		no-results-message="No matching issues."
		:dropdown-width="288"
		dropdown-align="right"
		:max-height="440"
		:clearable="false"
		:show-chevron="false"
		trigger-class="!px-2"
		fit-content
		trigger-type="base"
		trigger-size="sm"
		checkbox-position="right"
		@open="open"
		@close="close"
		@update:model-value="selectIssues"
	>
		<template #input-content>
			<span class="flex items-center gap-1">
				<PlusIcon class="size-4 shrink-0" aria-hidden="true" />
				Issue
			</span>
		</template>
	</MultiSelect>
</template>

<script setup lang="ts">
import { PlusIcon } from '@modrinth/assets'
import { MultiSelect, type MultiSelectItem } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { injectReviewMessages } from '~/providers/project-review/review-messages'
import { injectReviewPanels, type ReviewIssue } from '~/providers/project-review/review-panels'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import { injectReviewShortcuts } from '../shortcuts'

const emit = defineEmits<{ selected: [] }>()
const panels = injectReviewPanels()
const reviewMessages = injectReviewMessages()
const { pending } = injectReviewSubmission()

const selected = computed(() => panels.activeIssues.value.map(({ id }) => id))
const picker = ref<{ close: () => void; open: () => Promise<void> }>()
const shortcuts = injectReviewShortcuts()
const draft = ref([...selected.value])
const isOpen = ref(false)
const options = computed<MultiSelectItem<string>[]>(() => {
	const items: MultiSelectItem<string>[] = [{ value: 'add-custom-issue', label: 'Custom issue' }]
	const selectedIds = new Set(selected.value)
	const categories = new Map<string, ReviewIssue[]>()
	for (const issue of panels.availableIssues.value) {
		if (issue.custom || selectedIds.has(issue.id)) continue
		const group = categories.get(issue.category) ?? []
		group.push(issue)
		categories.set(issue.category, group)
	}
	const orderedCategories = [...categories].sort(
		([a], [b]) => Number(b === 'Project wide') - Number(a === 'Project wide'),
	)
	for (const [category, issues] of orderedCategories) {
		items.push({
			type: 'section-header',
			label: category,
			key: category,
			showSelectionActions: false,
		})
		for (const issue of issues) {
			items.push({
				value: issue.id,
				label: issue.title,
				searchTerms: [category],
				disabled: pending.value,
			})
		}
	}
	return items
})

function updateIssues(value: string[]) {
	if (pending.value) return
	const next = new Set(value)
	const current = new Set(selected.value)
	for (const id of current) {
		if (next.has(id)) continue
		panels.removeIssue(id)
		reviewMessages.resetIssueMessage(id)
	}
	for (const id of next) {
		if (!current.has(id)) panels.addIssue(id)
	}
}

function selectIssues(value: string[]) {
	if (value.includes('add-custom-issue')) {
		panels.addCustomIssue()
		draft.value = [...selected.value]
		emit('selected')
	} else {
		draft.value = value
		emit('selected')
	}
	picker.value?.close()
}

function open() {
	draft.value = [...selected.value]
	isOpen.value = true
}

function close() {
	isOpen.value = false
	const current = new Set(selected.value)
	if (draft.value.length !== current.size || draft.value.some((id) => !current.has(id))) {
		updateIssues(draft.value)
	}
}

watch(selected, (value) => {
	if (!isOpen.value) draft.value = [...value]
})
async function openPicker() {
	if (pending.value) return
	shortcuts.run('reveal-right')
	await picker.value?.open()
}
defineExpose({ openPicker })
</script>
