<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, PlusIcon, SearchIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { computed, nextTick, ref } from 'vue'

import { ButtonStyled, NewModal, StyledInput } from '#ui/components'
import { commonMessages } from '#ui/utils'

import { defineMessages, useVIntl } from '../../composables/i18n'

const props = defineProps<{
	groupId: string
	pending?: boolean
}>()

const emit = defineEmits<{
	(e: 'confirm', sha1s: string[]): void
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	defaultGroupTitle: {
		id: 'external-files.permissions-card.fallback-group-title',
		defaultMessage: 'Attribution group {id}',
	},
	addFilesModalTitle: {
		id: 'external-files.permissions-card.add-files-modal.title',
		defaultMessage: 'Add files to this group',
	},
	addFilesModalDescription: {
		id: 'external-files.permissions-card.add-files-modal.description',
		defaultMessage: 'Select any files that should be moved into this group.',
	},
	addFilesModalEmpty: {
		id: 'external-files.permissions-card.add-files-modal.empty',
		defaultMessage: 'There are no files in other groups that can be moved here.',
	},
	addFilesModalSearchPlaceholder: {
		id: 'external-files.permissions-card.add-files-modal.search-placeholder',
		defaultMessage: 'Search files...',
	},
	addFilesModalNoSearchResults: {
		id: 'external-files.permissions-card.add-files-modal.no-search-results',
		defaultMessage: 'No files match your search.',
	},
	addFilesModalConfirm: {
		id: 'external-files.permissions-card.add-files-modal.confirm',
		defaultMessage: '{count, plural, one {Add file} other {Add files}}',
	},
	addFilesModalSelectedCount: {
		id: 'external-files.permissions-card.add-files-modal.selected-count',
		defaultMessage: '{count, plural, one {# file selected} other {# files selected}}',
	},
})

type AssignableFileEntry = {
	sha1: string
	displayName: string
	sourceLabel: string
}

const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
const assignableEntries = ref<AssignableFileEntry[]>([])
const searchQuery = ref('')
const searchInputRef = ref<{ focus: () => void } | null>(null)
const selectedSha1s = ref<Set<string>>(new Set())
const selectionAnchorSha1 = ref<string | null>(null)

const filteredEntries = computed(() => {
	const q = searchQuery.value.trim().toLowerCase()
	if (!q) {
		return assignableEntries.value
	}
	return assignableEntries.value.filter(
		(e) =>
			e.displayName.toLowerCase().includes(q) ||
			e.sourceLabel.toLowerCase().includes(q) ||
			e.sha1.toLowerCase().includes(q),
	)
})

const selectedFileCount = computed(() => selectedSha1s.value.size)

function isFileSelected(sha1: string) {
	return selectedSha1s.value.has(sha1)
}

function toggleFileSelection(sha1: string, event: MouseEvent) {
	const next = new Set(selectedSha1s.value)
	const shouldSelect = !next.has(sha1)
	const entryIndex = filteredEntries.value.findIndex((entry) => entry.sha1 === sha1)
	const anchorIndex = filteredEntries.value.findIndex(
		(entry) => entry.sha1 === selectionAnchorSha1.value,
	)

	const sha1sToToggle =
		event.shiftKey && entryIndex !== -1 && anchorIndex !== -1
			? filteredEntries.value
					.slice(Math.min(entryIndex, anchorIndex), Math.max(entryIndex, anchorIndex) + 1)
					.map((entry) => entry.sha1)
			: [sha1]

	for (const sha1ToToggle of sha1sToToggle) {
		if (shouldSelect) {
			next.add(sha1ToToggle)
		} else {
			next.delete(sha1ToToggle)
		}
	}
	selectedSha1s.value = next
	selectionAnchorSha1.value = sha1
}

function clearSelectedFiles() {
	selectedSha1s.value = new Set()
	selectionAnchorSha1.value = null
}

function focusSearchOnOpen() {
	nextTick(() => {
		setTimeout(() => {
			searchInputRef.value?.focus()
		}, 75)
	})
}

function buildAssignableEntries(
	groups: Labrinth.Attribution.Internal.AttributionGroup[],
): AssignableFileEntry[] {
	const entries: AssignableFileEntry[] = []
	for (const g of groups) {
		if (g.id === props.groupId) {
			continue
		}
		const firstName = g.files?.[0]?.name
		const sourceLabel =
			g.flame_project?.title ??
			(firstName ? (firstName.split('/').pop() ?? firstName) : null) ??
			formatMessage(messages.defaultGroupTitle, { id: g.id })
		for (const f of g.files ?? []) {
			const displayName = f.name.split('/').pop() ?? f.name
			entries.push({ sha1: f.sha1, displayName, sourceLabel })
		}
	}
	entries.sort((a, b) => {
		const byName = a.displayName.localeCompare(b.displayName)
		if (byName !== 0) {
			return byName
		}
		return a.sourceLabel.localeCompare(b.sourceLabel)
	})
	return entries
}

function handleConfirm() {
	if (selectedFileCount.value === 0) {
		return
	}
	emit('confirm', [...selectedSha1s.value])
	hide()
}

function show(event: MouseEvent, groups: Labrinth.Attribution.Internal.AttributionGroup[]) {
	assignableEntries.value = buildAssignableEntries(groups)
	searchQuery.value = ''
	clearSelectedFiles()
	modalRef.value?.show(event)
}

function hide() {
	modalRef.value?.hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modalRef"
		:header="formatMessage(messages.addFilesModalTitle)"
		max-width="560px"
		:disable-close="pending"
		:on-show="focusSearchOnOpen"
		:on-hide="clearSelectedFiles"
		no-padding
	>
		<div class="flex flex-col gap-4 p-4 relative">
			<p class="text-secondary m-0">
				{{ formatMessage(messages.addFilesModalDescription) }}
			</p>
			<StyledInput
				ref="searchInputRef"
				v-model="searchQuery"
				type="text"
				autocomplete="off"
				:placeholder="formatMessage(messages.addFilesModalSearchPlaceholder)"
				:icon="SearchIcon"
				:disabled="pending"
				input-class="h-[40px]"
				class="sticky top-0"
				clearable
			/>
		</div>
		<div
			class="flex flex-col gap-2 bg-surface-2 overflow-y-auto overflow-x-hidden border-0 border-y border-solid border-surface-5 max-h-[calc(max(50vh,600px))]"
		>
			<div v-if="assignableEntries.length === 0" class="text-secondary text-sm p-4 m-0">
				{{ formatMessage(messages.addFilesModalEmpty) }}
			</div>
			<div v-else-if="filteredEntries.length === 0" class="text-secondary text-sm m-0 p-4">
				{{ formatMessage(messages.addFilesModalNoSearchResults) }}
			</div>
			<ul v-else class="p-0 m-0 list-none">
				<li
					v-for="entry in filteredEntries"
					:key="entry.sha1"
					:class="
						isFileSelected(entry.sha1)
							? 'border-brand bg-surface-4 hover:bg-surface-5'
							: 'hover:bg-surface-3 even:bg-surface-2.5'
					"
				>
					<button
						type="button"
						class="w-full text-left p-3 flex flex-col gap-1 appearance-none bg-transparent 3 transition-colors disabled:opacity-50"
						:disabled="pending"
						:aria-pressed="isFileSelected(entry.sha1)"
						@click="toggleFileSelection(entry.sha1, $event)"
					>
						<span class="flex gap-2 font-medium">
							<span
								class="size-4 shrink-0 rounded flex items-center justify-center border-[1px] border-solid"
								:class="
									isFileSelected(entry.sha1)
										? 'bg-brand border-button-border text-brand-inverted'
										: 'bg-surface-2 border-surface-5'
								"
							>
								<CheckIcon
									v-if="isFileSelected(entry.sha1)"
									class="size-3"
									aria-hidden="true"
									stroke-width="3"
								/>
							</span>
							<span class="flex flex-col gap-1 truncate">
								<span class="truncate">{{ entry.displayName }}</span>
								<span
									v-if="entry.sourceLabel !== entry.displayName"
									class="text-secondary text-xs truncate"
									>{{ entry.sourceLabel }}</span
								>
							</span>
						</span>
					</button>
				</li>
			</ul>
		</div>
		<template #actions>
			<div class="flex justify-between items-center gap-3 w-full pt-4">
				<p v-if="selectedFileCount > 0" class="text-secondary text-sm m-0">
					{{ formatMessage(messages.addFilesModalSelectedCount, { count: selectedFileCount }) }}
				</p>
				<div class="flex gap-2 ml-auto">
					<ButtonStyled type="outlined">
						<button type="button" :disabled="pending" @click="hide">
							<XIcon class="size-4 shrink-0" />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button
							type="button"
							:disabled="selectedFileCount === 0 || pending"
							@click="handleConfirm"
						>
							<SpinnerIcon v-if="pending" class="size-4 shrink-0 animate-spin" />
							<PlusIcon v-else class="size-4 shrink-0" />
							{{ formatMessage(messages.addFilesModalConfirm, { count: selectedFileCount }) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>
	</NewModal>
</template>
