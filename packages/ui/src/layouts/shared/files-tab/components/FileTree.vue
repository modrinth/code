<template>
	<div class="flex flex-col gap-0.5" :aria-label="formatMessage(messages.fileTree)">
		<template v-for="row in rows" :key="row.key">
			<div
				v-if="row.kind === 'loading'"
				class="flex h-9 items-center gap-2 text-sm text-secondary"
				:style="{ paddingLeft: `${2 + row.depth}rem` }"
			>
				<SpinnerIcon class="size-4 animate-spin" />
				{{ formatMessage(messages.loading) }}
			</div>
			<div
				v-else-if="row.kind === 'empty'"
				class="flex h-9 items-center text-sm italic text-secondary"
				:style="{ paddingLeft: `${2 + row.depth}rem` }"
			>
				{{ formatMessage(row.depth === 0 ? messages.emptyFolder : messages.emptySubfolder) }}
			</div>
			<FileTableRow
				v-else
				:count="row.item.count"
				:created="row.item.created"
				:modified="row.item.modified"
				:name="row.item.name"
				:path="row.item.path"
				:type="row.item.type"
				:size="row.item.size"
				:index="0"
				:is-last="false"
				:selected="ui.selectedItems.value.has(row.item.path)"
				:write-disabled="ui.isBusy.value || !!ctx.isReadOnly?.(row.item.path)"
				:write-disabled-tooltip="
					ctx.isReadOnly?.(row.item.path) ? ctx.readOnlyReason?.value : ui.busyTooltip.value
				"
				:show-details="false"
				:selection-within-action-menu="true"
				compact
				:depth="row.depth"
				:expanded="row.expanded"
				:active="row.path === activePath"
				:active-guide-level="activeGuideLevel(row.path)"
				@extract="() => ui.handleExtractItem(row.item)"
				@delete="() => ui.showDeleteModal(row.item)"
				@rename="() => ui.showRenameModal(row.item)"
				@download="() => ui.handleDownload(row.item)"
				@zip="() => ui.handleZip(row.item)"
				@move="() => ui.showMoveModal(row.item)"
				@move-direct-to="ui.handleDirectMove"
				@edit="() => ui.handleEditFile(row.item)"
				@navigate="() => openDirectory(row.path, row.item)"
				@open-in-new-tab="() => ui.handleOpenInNewTab(row.item)"
				@toggle-expand="() => toggleExpanded(row.path)"
				@hover="() => prefetch(row)"
				@contextmenu="ui.handleContextMenu"
				@toggle-select="() => ui.toggleItemSelection(row.item)"
			/>
		</template>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { computed, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { canOpenInFileEditor } from '#ui/utils/file-extensions'

import { directoryOf, normalizeFilePath } from '../composables/file-tabs'
import { injectFileBrowserUI } from '../providers/file-browser-ui'
import { injectFileManager } from '../providers/file-manager'
import type { FileItem } from '../types'
import FileTableRow from './FileTableRow.vue'

type TreeRow =
	| { kind: 'item'; key: string; item: FileItem; path: string; depth: number; expanded: boolean }
	| { kind: 'loading' | 'empty'; key: string; depth: number }

const ROOT_PATH = '/'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	fileTree: {
		id: 'files.tree.label',
		defaultMessage: 'File tree',
	},
	loading: {
		id: 'files.tree.loading',
		defaultMessage: 'Loading...',
	},
	emptyFolder: {
		id: 'files.tree.empty-folder',
		defaultMessage: 'This folder is empty',
	},
	emptySubfolder: {
		id: 'files.tree.empty-subfolder',
		defaultMessage: 'Empty',
	},
})

const ctx = injectFileManager()
const ui = injectFileBrowserUI()
const tree = ctx.directoryTree

const activePath = computed(() => ui.fileTabs.activeLocation.value.path)
const activeDirectory = computed(() => directoryOf(ui.fileTabs.activeLocation.value))
const activeDirectoryDepth = computed(
	() => activeDirectory.value.split('/').filter(Boolean).length,
)

function activeGuideLevel(path: string) {
	if (activeDirectoryDepth.value === 0) return undefined
	return path.startsWith(`${activeDirectory.value}/`) ? activeDirectoryDepth.value : undefined
}
const expandedPaths = computed(() => new Set(tree.expandedEntries.value))
const query = computed(() => ui.searchQuery.value.trim().toLowerCase())

function compareEntries(a: FileItem, b: FileItem) {
	if (a.type !== b.type) return a.type === 'directory' ? -1 : 1
	return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' })
}

function collectRows(path: string, depth: number): TreeRow[] {
	const entries = tree.get(path)
	const items = entries.items.value

	if (items.length === 0) {
		if (entries.isLoading.value) return [{ kind: 'loading', key: `loading:${path}`, depth }]
		return query.value ? [] : [{ kind: 'empty', key: `empty:${path}`, depth }]
	}

	const rows: TreeRow[] = []
	for (const item of [...items].sort(compareEntries)) {
		const itemPath = normalizeFilePath(item.path)
		const expanded = item.type === 'directory' && expandedPaths.value.has(itemPath)
		const children = expanded ? collectRows(itemPath, depth + 1) : []

		if (query.value) {
			const matches = item.name.toLowerCase().includes(query.value)
			const hasMatchingChild = children.some((child) => child.kind === 'item')
			if (!matches && !hasMatchingChild) continue
		}

		rows.push({ kind: 'item', key: itemPath, item, path: itemPath, depth, expanded }, ...children)
	}
	return rows
}

const rows = computed(() => collectRows(ROOT_PATH, 0))

function setExpanded(path: string, expanded: boolean) {
	const current = tree.expandedEntries.value
	if (expanded === current.includes(path)) return
	tree.expandedEntries.value = expanded
		? [...current, path]
		: current.filter((entry) => entry !== path)
}

function toggleExpanded(path: string) {
	setExpanded(path, !expandedPaths.value.has(path))
}

function openDirectory(path: string, item: FileItem) {
	setExpanded(path, true)
	ui.handleNavigateToFolder(item)
}

function prefetch(row: Extract<TreeRow, { kind: 'item' }>) {
	if (row.item.type === 'directory') {
		tree.prefetch(row.path)
	} else if (canOpenInFileEditor(row.item.name)) {
		ctx.prefetchFile?.(row.item.path)
	}
}

watch(
	() => directoryOf(ui.fileTabs.activeLocation.value),
	(directory) => {
		const segments = directory.split('/').filter(Boolean)
		const missing = segments
			.map((_, index) => `/${segments.slice(0, index + 1).join('/')}`)
			.filter((path) => !expandedPaths.value.has(path))
		if (missing.length > 0) tree.expandedEntries.value = [...tree.expandedEntries.value, ...missing]
	},
	{ immediate: true },
)
</script>
