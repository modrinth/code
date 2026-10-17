import type { ComputedRef, Ref } from 'vue'
import { computed, effectScope, watch } from 'vue'

import { isSameInfo, parentInfoFrom } from '#ui/layouts/shared/files-tab/utils.ts'

import type { DirectoryTree, FileInfo } from '../providers/file-manager'
import type { FileItem, FileSortField } from '../types'
import { useFileSearch } from './file-search'
import type { FileListingSelection, useFileSelection } from './file-selection'
import { useFileSorting } from './file-sorting'

/**
 * What one tab shows of the file system: its location, and when that is a directory, the
 * listing with its own search, sort and select-all state.
 */
export interface DirectoryView extends FileListingSelection {
	/** The tab's location, which may be a file (whose directory is then listed behind it). */
	location: ComputedRef<FileInfo>
	directory: ComputedRef<FileInfo<'directory'>>
	isFile: ComputedRef<boolean>
	items: ComputedRef<FileItem[]>
	/** The listing after search and sorting. */
	filteredItems: ComputedRef<FileItem[]>
	loading: ComputedRef<boolean>
	error: ComputedRef<Error | null>
	searchQuery: Ref<string>
	sortField: Ref<FileSortField>
	sortDesc: Ref<boolean>
	handleSort: (field: FileSortField) => void
}

export interface DirectoryViewOptions {
	location: () => FileInfo
	directoryTree: DirectoryTree
	selection: ReturnType<typeof useFileSelection>
}

function directoryOf(location: FileInfo): FileInfo<'directory'> {
	return (
		location.type === 'directory' ? location : parentInfoFrom(location)
	) as FileInfo<'directory'>
}

/** A tab's view of the file system. Search and sort reset whenever it moves to another directory. */
export function useDirectoryView(options: DirectoryViewOptions): DirectoryView {
	const location = computed(options.location)
	const directory = computed<FileInfo<'directory'>>((previous) => {
		const next = directoryOf(location.value)
		return previous && isSameInfo(previous, next) ? previous : next
	})
	const listing = computed(() => options.directoryTree.get(directory.value))

	const items = computed(() => listing.value.data.value)
	const { searchQuery, searchedItems } = useFileSearch(items)
	const { sortField, sortDesc, handleSort, sortedItems, resetSort } = useFileSorting(searchedItems)

	watch(directory, () => {
		searchQuery.value = ''
		resetSort()
	})

	return {
		location,
		directory,
		isFile: computed(() => location.value.type === 'file'),
		items,
		filteredItems: sortedItems,
		loading: computed(() => listing.value.isLoading.value),
		error: computed(() => listing.value.loadError.value ?? null),
		searchQuery,
		sortField,
		sortDesc,
		handleSort,
		...options.selection.forListing(sortedItems),
	}
}

/**
 * One {@link DirectoryView} per tab, created on first use and stopped once the tab is gone.
 * Each runs in its own effect scope, so it outlives the components showing it.
 */
export function useDirectoryViews(
	options: Omit<DirectoryViewOptions, 'location'> & {
		locationOf: (tabId: string) => FileInfo | undefined
		tabIds: () => string[]
	},
) {
	const views = new Map<string, { view: DirectoryView; stop: () => void }>()

	function viewFor(tabId: string): DirectoryView {
		const existing = views.get(tabId)
		if (existing) return existing.view
		const scope = effectScope(true)
		let lastLocation: FileInfo = { name: '', path: '/', type: 'directory' }
		const view = scope.run(() =>
			useDirectoryView({
				...options,
				location: () => {
					const location = options.locationOf(tabId)
					if (location) lastLocation = location
					return lastLocation
				},
			}),
		)!
		views.set(tabId, { view, stop: () => scope.stop() })
		return view
	}

	watch(options.tabIds, (ids) => {
		for (const [tabId, entry] of views) {
			if (ids.includes(tabId)) continue
			entry.stop()
			views.delete(tabId)
		}
	})

	return { viewFor }
}
