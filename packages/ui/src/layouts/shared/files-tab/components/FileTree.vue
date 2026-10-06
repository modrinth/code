<template>
	<div class="flex flex-col gap-0.5" :aria-label="formatMessage(messages.fileTree)">
		<template v-for="(row, index) in rows" :key="row.key">
			<div
				v-if="row.type === 'loading'"
				class="flex h-9 items-center gap-2 text-sm text-secondary"
				:style="{ paddingLeft: `${2 + row.depth}rem` }"
			>
				<SpinnerIcon class="size-4 animate-spin" />
				{{ formatMessage(messages.loading) }}
			</div>
			<div
				v-else-if="row.type === 'empty'"
				class="flex h-9 items-center text-sm italic text-secondary"
				:style="{ paddingLeft: `${2 + row.depth}rem` }"
			>
				{{ formatMessage(row.depth === 0 ? messages.emptyFolder : messages.emptySubfolder) }}
			</div>
			<FileRow
				v-else-if="row.type === 'item'"
				:file="row.item"
				:index="index"
				:is-last="index + 1 == rows.length"
				:selection-within-action-menu="true"
				compact
				:depth="row.depth"
				:expanded="row.expanded"
				:expandable="row.expandable"
				is-tree-row
				:active="row.path === activePath"
				:active-guide-level="activeGuideLevel(row.path)"
				@navigate="openDirectory"
				@toggle-expand="toggleExpanded"
				@hover="prefetch"
			/>
		</template>
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { computed, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { infoFrom, parentInfoFrom } from '#ui/layouts/shared/files-tab/utils.ts'
import { canOpenInFileEditor } from '#ui/utils/file-extensions'

import { injectFileBrowserUI } from '../providers/file-browser-ui'
import type { FileItem } from '../types'
import FileRow from './FileRow.vue'

type TreeRow =
	| {
			type: 'item'
			key: string
			item: FileItem
			path: string
			depth: number
			expanded: boolean
			expandable: boolean
	  }
	| { type: 'loading' | 'empty'; key: string; depth: number }

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

const ui = injectFileBrowserUI()
const tree = ui.directoryTree

const activePath = computed(() => ui.fileTabs.activeLocation.value.path)
const activeDirectory = computed(() => parentInfoFrom(ui.fileTabs.activeLocation.value))
const activeDirectoryDepth = computed(
	() => activeDirectory.value.path.split('/').filter(Boolean).length,
)

function activeGuideLevel(path: string) {
	if (activeDirectoryDepth.value === 0) return undefined
	return path.startsWith(`${activeDirectory.value.path}/`) ? activeDirectoryDepth.value : undefined
}
const expandedPaths = computed(() => new Set(tree.expandedEntries.value))
const query = computed(() => ui.searchQuery.value.trim().toLowerCase())

/** Directories known to be empty can't be expanded; an unknown count stays expandable. */
function isExpandable(item: FileItem) {
	return item.type === 'directory' && item.count !== 0
}

const nameCollator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' })

function compareEntries(a: FileItem, b: FileItem) {
	if (a.type !== b.type) return a.type === 'directory' ? -1 : 1
	return nameCollator.compare(a.name, b.name)
}

function collectRows(path: string, depth: number): TreeRow[] {
	const entries = tree.get(infoFrom(path))
	const items = entries.data.value

	if (items.length === 0) {
		if (entries.isLoading.value) return [{ type: 'loading', key: `loading:${path}`, depth }]
		return query.value ? [] : [{ type: 'empty', key: `empty:${path}`, depth }]
	}

	const rows: TreeRow[] = []
	for (const item of [...items].sort(compareEntries)) {
		const expandable = isExpandable(item)
		const expanded = expandable && expandedPaths.value.has(item.path)
		const children = expanded ? collectRows(item.path, depth + 1) : []

		if (query.value) {
			const matches = item.name.toLowerCase().includes(query.value)
			const hasMatchingChild = children.some((child) => child.type === 'item')
			if (!matches && !hasMatchingChild) continue
		}

		rows.push(
			{ type: 'item', key: item.path, item, path: item.path, depth, expanded, expandable },
			...children,
		)
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

function toggleExpanded(item: FileItem) {
	setExpanded(item.path, !expandedPaths.value.has(item.path))
}

function openDirectory(item: FileItem) {
	if (isExpandable(item)) setExpanded(item.path, true)
	ui.navigateTo(item)
}

function prefetch(item: FileItem) {
	if (item.type === 'directory' || canOpenInFileEditor(item.name)) {
		tree.prefetch(item)
	}
}

watch(
	() => parentInfoFrom(ui.fileTabs.activeLocation.value),
	(directory) => {
		const segments = directory.path.split('/').filter(Boolean)
		const missing = segments
			.map((_, index) => `/${segments.slice(0, index + 1).join('/')}`)
			.filter((path) => !expandedPaths.value.has(path))
		if (missing.length > 0) tree.expandedEntries.value = [...tree.expandedEntries.value, ...missing]
	},
	{ immediate: true },
)
</script>
