<script setup lang="ts">
import {
	ChevronDownIcon,
	ChevronRightIcon,
	PlusIcon,
	SearchIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Button,
	ConfirmLeaveModal,
	IconButton,
	Input,
	InlineValidationMessage,
	type SortDirection,
	Table,
	type TableColumn,
	UnsavedChangesPopup,
	usePageLeaveSafety,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import ArchiveModal from './components/ArchiveModal.vue'
import BulkImportModal from './components/BulkImportModal.vue'
import JsonEditor from './components/JsonEditor.vue'
import { type ArchiveDraft, fromDraft } from './types'
import { useArchiveEditor } from './use-archive-editor'

const {
	archives,
	archiveJson,
	validationErrors,
	loading,
	saving,
	busy,
	loadError,
	saveError,
	originalSnapshot,
	currentSnapshot,
	hasChanges,
	load,
	upsert,
	remove,
	save,
	reset,
} = useArchiveEditor()
const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)
const archiveModal = ref<InstanceType<typeof ArchiveModal>>()
const bulkModal = ref<InstanceType<typeof BulkImportModal>>()
const expandedIds = ref<string[]>([])
const search = ref('')
const sortColumn = ref<string | undefined>('name')
const sortDirection = ref<SortDirection>('asc')
const columns: TableColumn[] = [
	{ key: 'expand', width: '48px' },
	{ key: 'name', label: 'Project name', enableSorting: true },
	{ key: 'source', label: 'Source / type', width: '20%' },
	{ key: 'license', label: 'License', width: '16%', enableSorting: true },
	{ key: 'fileCount', label: 'Files', width: '10%', enableSorting: true },
	{ key: 'delete', width: '64px', align: 'right' },
]
const rows = computed(() => {
	const query = search.value.trim().toLowerCase()
	return archives.value
		.map((archive) => ({
			...archive,
			fileCount: archive.files.length,
			source: archive.modrinthMetadata
				? `Modrinth / ${archive.modrinthMetadata.projectType}`
				: archive.minecraftJson.trim()
					? 'Minecraft'
					: 'Other',
			expand: '',
			delete: '',
		}))
		.filter((archive) => !query || JSON.stringify(archive).toLowerCase().includes(query))
		.sort((a, b) => {
			const direction = sortDirection.value === 'asc' ? 1 : -1
			if (sortColumn.value === 'fileCount') return (a.fileCount - b.fileCount) * direction
			const key = sortColumn.value === 'license' ? 'license' : 'name'
			return a[key].localeCompare(b[key], 'en') * direction
		})
})
function toggleArchive(archive: ArchiveDraft) {
	if (busy.value) return
	if (expandedIds.value.includes(archive.id)) {
		expandedIds.value = expandedIds.value.filter((id) => id !== archive.id)
	} else {
		if (archiveJson.value[archive.id] === undefined) {
			const saved = archives.value.find((record) => record.id === archive.id)
			if (!saved) return
			archiveJson.value[archive.id] = JSON.stringify(fromDraft(saved), null, '\t')
		}
		expandedIds.value.push(archive.id)
	}
}
function closeEditor(id: string) {
	expandedIds.value = expandedIds.value.filter((expandedId) => expandedId !== id)
	delete archiveJson.value[id]
}
function deleteArchive(id: string) {
	if (busy.value) return
	remove(id)
	closeEditor(id)
}
async function saveAll() {
	await save()
	const invalidIds = Object.keys(validationErrors.value)
	if (invalidIds.length) {
		search.value = ''
		expandedIds.value = [...new Set([...expandedIds.value, ...invalidIds])]
	}
}
function resetAll() {
	reset()
	expandedIds.value = expandedIds.value.filter((id) =>
		archives.value.some((archive) => archive.id === id),
	)
}
function createArchive() {
	archiveModal.value?.show()
}
</script>

<template>
	<div class="flex flex-col gap-4">
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<ArchiveModal ref="archiveModal" :persist="upsert" :saving="saving" :save-error="saveError" />
		<BulkImportModal
			ref="bulkModal"
			:existing-ids="archives.map((archive) => archive.id)"
			:persist="upsert"
			:saving="saving"
			:save-error="saveError"
		/>
		<div class="flex flex-wrap items-center justify-between gap-4">
			<h2 class="m-0 text-2xl font-semibold">Copyright archives</h2>
			<div class="flex flex-wrap items-center gap-2">
				<Input
					v-model="search"
					:icon="SearchIcon"
					type="search"
					placeholder="Search archives or hashes..."
					clearable
					wrapper-class="w-full sm:w-72"
				/>
				<Button type="outlined" :disabled="busy" @click="bulkModal?.show()">Bulk import</Button>
				<Button type="colored" color="brand" :disabled="busy" @click="createArchive"
					><PlusIcon />New archive</Button
				>
			</div>
		</div>

		<div v-if="loadError" role="alert" class="flex flex-col items-start gap-2">
			<p class="m-0 text-red">Could not load archives: {{ loadError }}</p>
			<Button type="outlined" @click="load">Retry</Button>
		</div>
		<template v-else>
			<Table
				v-model:sort-column="sortColumn"
				v-model:sort-direction="sortDirection"
				:columns="columns"
				:data="rows"
				row-key="id"
				table-min-width="800px"
				:row-clickable="!busy"
				:row-below-visible="(row) => expandedIds.includes(row.id)"
				row-class="cursor-pointer hover:brightness-125"
				@row-click="toggleArchive"
			>
				<template #cell-expand="{ row }">
					<ChevronDownIcon v-if="expandedIds.includes(row.id)" class="size-5" aria-hidden="true" />
					<ChevronRightIcon v-else class="size-5" aria-hidden="true" />
				</template>
				<template #cell-name="{ row }">
					<div class="flex flex-col gap-1 py-3 pr-3">
						<button
							type="button"
							class="line-clamp-2 cursor-pointer border-0 bg-transparent p-0 text-left font-medium text-primary"
							:aria-expanded="expandedIds.includes(row.id)"
							:aria-controls="`archive-json-${row.id}`"
							:disabled="busy"
						>
							{{ row.name || 'Untitled archive' }}
						</button>
					</div>
				</template>
				<template #cell-source="{ row }"
					><span class="break-words pr-2">{{ row.source }}</span></template
				>
				<template #cell-license="{ row }"
					><span class="break-words pr-2">{{ row.license || '—' }}</span></template
				>
				<template #cell-delete="{ row }">
					<div class="flex justify-end" data-no-row-click>
						<IconButton
							type="outlined"
							:disabled="busy"
							class="[&:enabled:hover>svg]:!text-red"
							:label="`Delete ${row.name}`"
							@click="deleteArchive(row.id)"
						>
							<TrashIcon class="transition-colors duration-150 ease-out" aria-hidden="true" />
						</IconButton>
					</div>
				</template>
				<template #row-below="{ row }">
					<div :id="`archive-json-${row.id}`" class="cursor-auto p-4" data-no-row-click>
						<div class="flex flex-col gap-3">
							<span class="font-semibold">Archive JSON</span>
							<JsonEditor
								v-model="archiveJson[row.id]"
								:label="`${row.name} JSON`"
								:disabled="saving"
								:error="!!validationErrors[row.id]"
							/>
							<InlineValidationMessage :message="validationErrors[row.id]" />
						</div>
					</div>
				</template>
				<template #empty-state>
					<div class="flex h-64 items-center justify-center text-secondary">
						{{
							loading
								? 'Loading archives...'
								: search.trim()
									? 'No matching archives.'
									: 'No archives yet. Create an archive or import JSON to add project archives.'
						}}
					</div>
				</template>
			</Table>
		</template>
		<InlineValidationMessage :message="saveError" />
		<UnsavedChangesPopup
			:original="{ archives: originalSnapshot }"
			:modified="{ archives: currentSnapshot }"
			:saving="saving"
			:can-save="!busy"
			save-label="Save"
			@save="saveAll"
			@reset="resetAll"
		/>
	</div>
</template>
