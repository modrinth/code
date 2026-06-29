<template>
	<div class="flex w-full min-w-0 flex-col gap-2">
		<nav
			:aria-label="formatMessage(messages.breadcrumbNavigation)"
			class="m-0 flex w-full min-w-0 flex-shrink items-center justify-between gap-2 p-0 text-contrast"
		>
			<ol class="m-0 flex min-w-0 flex-shrink list-none items-center p-0">
				<li class="mr-1 flex-shrink-0">
					<ButtonStyled circular size="small">
						<button
							v-tooltip="formatMessage(messages.backToHome)"
							type="button"
							class="bg-surface-4 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand !shadow-none !size-8"
							:class="{ 'cursor-not-allowed opacity-60': isHomePath }"
							:aria-label="formatMessage(messages.backToHome)"
							:disabled="isHomePath"
							@click="navigateTo('')"
						>
							<HomeIcon />
							<span class="sr-only">{{ formatMessage(messages.home) }}</span>
						</button>
					</ButtonStyled>
				</li>
				<li class="m-0 min-w-0 flex-shrink p-0">
					<ol class="m-0 flex min-w-0 flex-shrink items-center overflow-hidden p-0">
						<li
							v-for="(breadcrumb, index) in breadcrumbs"
							:key="breadcrumb.path"
							class="relative flex shrink-0 items-center text-sm"
						>
							<div class="flex shrink-0 items-center">
								<ButtonStyled type="transparent" size="small">
									<button
										type="button"
										class="cursor-pointer whitespace-nowrap text-sm font-semibold focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand"
										:aria-current="index === breadcrumbs.length - 1 ? 'location' : undefined"
										:class="{ '!text-contrast': index === breadcrumbs.length - 1 }"
										@click="navigateTo(breadcrumb.path)"
									>
										{{ breadcrumb.name }}
									</button>
								</ButtonStyled>
								<ChevronRightIcon
									v-if="index < breadcrumbs.length - 1"
									class="size-4 flex-shrink-0 text-secondary"
									aria-hidden="true"
								/>
							</div>
						</li>
					</ol>
				</li>
			</ol>
			<div v-if="$slots.actions" class="ml-auto flex shrink-0 items-center">
				<slot name="actions" />
			</div>
		</nav>

		<div
			class="flex w-full min-w-0 flex-col rounded-[20px] border border-solid border-surface-4 shadow-sm overflow-clip"
		>
			<div
				class="flex h-10 w-full min-w-0 select-none flex-row items-center justify-between bg-surface-3 px-3 text-sm font-medium"
			>
				<div class="flex min-w-0 flex-1 items-center gap-2">
					<Checkbox
						:model-value="allVisibleSelected"
						:indeterminate="someVisibleSelected && !allVisibleSelected"
						:disabled="visibleSelectableEntries.length === 0"
						@update:model-value="toggleAllVisible"
					/>
					<button
						type="button"
						class="flex min-w-0 appearance-none items-center gap-1.5 border-0 bg-transparent p-0 font-semibold hover:text-primary"
						:class="sortField === 'name' ? 'text-contrast' : 'text-secondary'"
						@click="handleSort('name')"
					>
						<span class="min-w-0 truncate">{{ formatMessage(messages.name) }}</span>
						<ChevronUpIcon
							v-if="sortField === 'name' && !sortDesc"
							class="size-4 shrink-0"
							aria-hidden="true"
						/>
						<ChevronDownIcon
							v-if="sortField === 'name' && sortDesc"
							class="size-4 shrink-0"
							aria-hidden="true"
						/>
					</button>
				</div>
				<div class="ml-2 flex shrink-0 items-center gap-4">
					<button
						type="button"
						class="hidden w-[92px] appearance-none items-center gap-1 border-0 bg-transparent p-0 text-left font-semibold hover:text-primary sm:flex"
						:class="sortField === 'size' ? 'text-contrast' : 'text-secondary'"
						@click="handleSort('size')"
					>
						<span>{{ formatMessage(messages.size) }}</span>
						<ChevronUpIcon
							v-if="sortField === 'size' && !sortDesc"
							class="size-4 shrink-0"
							aria-hidden="true"
						/>
						<ChevronDownIcon
							v-if="sortField === 'size' && sortDesc"
							class="size-4 shrink-0"
							aria-hidden="true"
						/>
					</button>
					<button
						type="button"
						class="hidden w-[132px] appearance-none items-center gap-1 border-0 bg-transparent p-0 text-left font-semibold hover:text-primary sm:flex"
						:class="sortField === 'modified' ? 'text-contrast' : 'text-secondary'"
						@click="handleSort('modified')"
					>
						<span>{{ formatMessage(messages.modified) }}</span>
						<ChevronUpIcon
							v-if="sortField === 'modified' && !sortDesc"
							class="size-4 shrink-0"
							aria-hidden="true"
						/>
						<ChevronDownIcon
							v-if="sortField === 'modified' && sortDesc"
							class="size-4 shrink-0"
							aria-hidden="true"
						/>
					</button>
					<span class="size-4 shrink-0" aria-hidden="true" />
				</div>
			</div>
			<div
				v-if="entries.length === 0"
				class="flex items-center gap-2 bg-surface-2 px-3 py-2 text-sm text-secondary"
			>
				<FileIcon class="size-4 shrink-0" />
				<span>{{ formatMessage(messages.emptyFolderTitle) }}</span>
			</div>
			<div
				v-for="(entry, i) in entries"
				:key="`${entry.type}:${entry.path}`"
				role="button"
				tabindex="0"
				class="group flex w-full min-w-0 select-none items-center gap-2 border-0 border-t border-solid border-surface-4 px-3 py-2 text-left first:border-t-0 focus:!outline-none"
				:class="[
					i % 2 === 0 ? 'bg-surface-2' : 'bg-surface-1.5',
					entry.disabled && entry.type === 'file'
						? 'cursor-not-allowed opacity-50'
						: 'cursor-pointer hover:bg-surface-2.5',
				]"
				@click="selectEntry(entry)"
				@keydown="(event) => event.key === 'Enter' && selectEntry(entry)"
			>
				<Checkbox
					class="shrink-0"
					:model-value="entry.checked"
					:indeterminate="entry.indeterminate"
					:disabled="entry.disabled"
					:description="entry.name"
					@click.stop
					@update:model-value="toggleEntry(entry, $event)"
				/>
				<div class="flex size-4 shrink-0 items-center justify-center text-secondary">
					<component
						:is="entry.icon"
						class="size-4 group-hover:text-contrast group-focus:text-contrast"
					/>
				</div>
				<span
					:ref="(element) => setEntryNameRef(entry.path, element)"
					v-tooltip="truncatedTooltip(entryNameRefs[entry.path], entry.name)"
					class="min-w-0 flex-1 truncate text-sm font-medium text-primary group-hover:text-contrast group-focus:text-contrast"
				>
					{{ entry.name }}
				</span>
				<div class="ml-2 flex shrink-0 items-center gap-4">
					<span class="hidden w-[92px] truncate text-left text-sm text-secondary sm:block">
						{{ formatSize(entry) }}
					</span>
					<span class="hidden w-[132px] truncate text-left text-sm text-secondary sm:block">
						{{ formatModified(entry) }}
					</span>
					<ChevronRightIcon
						class="size-4 shrink-0 text-secondary group-hover:text-contrast group-focus:text-contrast"
						:class="{ invisible: entry.type !== 'directory' }"
						aria-hidden="true"
					/>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	ChevronDownIcon,
	ChevronRightIcon,
	ChevronUpIcon,
	FileIcon,
	HomeIcon,
} from '@modrinth/assets'
import { type Component, type ComponentPublicInstance, computed, ref, watch } from 'vue'

import { useFormatBytes } from '../../composables/format-bytes'
import { useFormatDateTime } from '../../composables/format-date-time'
import { defineMessages, useVIntl } from '../../composables/i18n'
import { getDirectoryIcon, getFileIcon } from '../../utils/auto-icons'
import { truncatedTooltip } from '../../utils/truncate'
import ButtonStyled from './ButtonStyled.vue'
import Checkbox from './Checkbox.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	name: {
		id: 'files.table-header.name',
		defaultMessage: 'Name',
	},
	size: {
		id: 'files.table-header.size',
		defaultMessage: 'Size',
	},
	modified: {
		id: 'files.table-header.modified',
		defaultMessage: 'Modified',
	},
	itemCount: {
		id: 'files.row.item-count',
		defaultMessage: '{count, plural, one {# item} other {# items}}',
	},
	breadcrumbNavigation: {
		id: 'files.navbar.breadcrumb-navigation',
		defaultMessage: 'Breadcrumb navigation',
	},
	backToHome: {
		id: 'files.navbar.back-to-home',
		defaultMessage: 'Back to home',
	},
	home: {
		id: 'files.navbar.home',
		defaultMessage: 'Home',
	},
	emptyFolderTitle: {
		id: 'files.layout.empty-folder-title',
		defaultMessage: 'This folder is empty',
	},
})

export type FileTreeSelectItem = {
	path: string
	type?: 'directory' | 'file'
	disabled?: boolean
	size?: number
	modified?: number
	count?: number
}

type NormalizedFileTreeSelectItem = FileTreeSelectItem & {
	name: string
	normalizedPath: string
}

type FileTreeSelectEntry = {
	path: string
	name: string
	type: 'directory' | 'file'
	icon: Component
	checked: boolean
	indeterminate: boolean
	disabled: boolean
	size?: number
	modified?: number
	count?: number
	item?: NormalizedFileTreeSelectItem
}

type FileTreeSelectSortField = 'name' | 'size' | 'modified'

const formatBytes = useFormatBytes()
const formatDateTime = useFormatDateTime({
	year: '2-digit',
	month: '2-digit',
	day: '2-digit',
	hour: 'numeric',
	minute: 'numeric',
})

const props = withDefaults(
	defineProps<{
		items: FileTreeSelectItem[]
		modelValue: string[]
	}>(),
	{
		items: () => [],
		modelValue: () => [],
	},
)

const emit = defineEmits<{
	(e: 'update:modelValue', value: string[]): void
	(e: 'navigate', path: string): void
}>()

const currentPath = ref('')
const sortField = ref<FileTreeSelectSortField>('name')
const sortDesc = ref(false)
const entryNameRefs = ref<Record<string, HTMLElement | null>>({})
const isHomePath = computed(() => currentPath.value === '')

const normalizedItems = computed(() => {
	const items = new Map<string, NormalizedFileTreeSelectItem>()
	for (const item of props.items) {
		const normalizedPath = normalizePath(item.path)
		if (!normalizedPath) continue
		items.set(normalizedPath, {
			...item,
			path: item.path,
			name: getName(normalizedPath),
			normalizedPath,
		})
	}
	return [...items.values()]
})

const selectedPaths = computed(() => new Set(props.modelValue.map((path) => normalizePath(path))))

const folderPaths = computed(() => {
	const paths = new Set<string>()
	for (const item of normalizedItems.value) {
		const segments = item.normalizedPath.split('/')
		for (let i = 1; i < segments.length; i++) {
			paths.add(segments.slice(0, i).join('/'))
		}
		if (item.type === 'directory') {
			paths.add(item.normalizedPath)
		}
	}
	return paths
})

const breadcrumbs = computed(() => {
	const segments = currentPath.value.split('/').filter(Boolean)
	return segments.map((name, index) => ({
		name,
		path: segments.slice(0, index + 1).join('/'),
	}))
})

const entries = computed<FileTreeSelectEntry[]>(() => {
	const directories = new Map<string, FileTreeSelectEntry>()
	const files: FileTreeSelectEntry[] = []
	const currentSegments = currentPath.value ? currentPath.value.split('/') : []

	for (const item of normalizedItems.value) {
		const segments = item.normalizedPath.split('/')
		if (!isInCurrentPath(segments, currentSegments)) continue

		const remaining = segments.slice(currentSegments.length)
		if (remaining.length > 1) {
			const directoryName = remaining[0]
			const directoryPath = [...currentSegments, directoryName].join('/')
			if (!directories.has(directoryPath)) {
				directories.set(directoryPath, buildDirectoryEntry(directoryPath, directoryName))
			}
		} else if (remaining.length === 1) {
			if (item.type === 'directory') {
				directories.set(
					item.normalizedPath,
					buildDirectoryEntry(item.normalizedPath, item.name, item),
				)
			} else {
				files.push(buildFileEntry(item))
			}
		}
	}

	return [...directories.values(), ...files].sort(compareEntries)
})

const visibleSelectableEntries = computed(() => entries.value.filter((entry) => !entry.disabled))

const visibleSelectablePaths = computed(() => {
	const paths = new Set<string>()
	for (const entry of visibleSelectableEntries.value) {
		for (const path of getEntrySelectablePaths(entry)) {
			paths.add(path)
		}
	}
	return [...paths]
})

const allVisibleSelected = computed(
	() =>
		visibleSelectablePaths.value.length > 0 &&
		visibleSelectablePaths.value.every((path) => selectedPaths.value.has(path)),
)

const someVisibleSelected = computed(() =>
	visibleSelectablePaths.value.some((path) => selectedPaths.value.has(path)),
)

watch(
	normalizedItems,
	() => {
		if (currentPath.value && !folderPaths.value.has(currentPath.value)) {
			currentPath.value = ''
		}
	},
	{ immediate: true },
)

function normalizePath(path: string) {
	return path.replaceAll('\\', '/').split('/').filter(Boolean).join('/')
}

function getName(path: string) {
	return path.split('/').pop() ?? path
}

function isInCurrentPath(segments: string[], currentSegments: string[]) {
	if (segments.length <= currentSegments.length) return false
	return currentSegments.every((segment, index) => segments[index] === segment)
}

function handleSort(field: FileTreeSelectSortField) {
	if (sortField.value === field) {
		sortDesc.value = !sortDesc.value
	} else {
		sortField.value = field
		sortDesc.value = false
	}
}

function compareEntries(a: FileTreeSelectEntry, b: FileTreeSelectEntry) {
	if (a.type !== b.type) return a.type === 'directory' ? -1 : 1

	switch (sortField.value) {
		case 'modified':
			return sortDesc.value
				? getModifiedSortValue(a) - getModifiedSortValue(b)
				: getModifiedSortValue(b) - getModifiedSortValue(a)
		case 'size':
			return sortDesc.value
				? getSizeSortValue(a) - getSizeSortValue(b)
				: getSizeSortValue(b) - getSizeSortValue(a)
		default:
			return sortDesc.value
				? b.name.localeCompare(a.name, undefined, { numeric: true, sensitivity: 'base' })
				: a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' })
	}
}

function getSizeSortValue(entry: FileTreeSelectEntry) {
	return entry.type === 'directory' ? (entry.count ?? 0) : (entry.size ?? 0)
}

function getModifiedSortValue(entry: FileTreeSelectEntry) {
	return entry.modified ?? 0
}

function getFolderDescendants(path: string) {
	return normalizedItems.value.filter((item) => item.normalizedPath.startsWith(`${path}/`))
}

function getFolderChildCount(path: string) {
	const children = new Set<string>()
	const prefix = `${path}/`
	for (const item of normalizedItems.value) {
		if (!item.normalizedPath.startsWith(prefix)) continue

		const relativePath = item.normalizedPath.slice(prefix.length)
		const [childName] = relativePath.split('/')
		if (childName) {
			children.add(childName)
		}
	}
	return children.size
}

function getLatestModified(items: NormalizedFileTreeSelectItem[]) {
	const modified = items
		.map((item) => item.modified)
		.filter((value): value is number => typeof value === 'number' && Number.isFinite(value))

	if (modified.length === 0) return undefined
	return Math.max(...modified)
}

function buildDirectoryEntry(
	path: string,
	name: string,
	item?: NormalizedFileTreeSelectItem,
): FileTreeSelectEntry {
	const descendants = getFolderDescendants(path).filter((item) => !item.disabled)
	const selectedCount = descendants.filter((item) =>
		selectedPaths.value.has(item.normalizedPath),
	).length
	const selected = selectedPaths.value.has(path)

	return {
		path,
		name,
		type: 'directory',
		icon: getDirectoryIcon(name),
		checked: selected || (descendants.length > 0 && selectedCount === descendants.length),
		indeterminate: !selected && selectedCount > 0 && selectedCount < descendants.length,
		disabled: item?.disabled ?? descendants.length === 0,
		modified: item?.modified ?? getLatestModified(descendants),
		count: item?.count ?? getFolderChildCount(path),
		item,
	}
}

function buildFileEntry(item: NormalizedFileTreeSelectItem): FileTreeSelectEntry {
	return {
		path: item.normalizedPath,
		name: item.name,
		type: 'file',
		icon: getFileIcon(item.name),
		checked: selectedPaths.value.has(item.normalizedPath),
		indeterminate: false,
		disabled: item.disabled ?? false,
		size: item.size,
		modified: item.modified,
		item,
	}
}

function navigateTo(path: string) {
	currentPath.value = path
	emit('navigate', path)
}

function setEntryNameRef(path: string, element: Element | ComponentPublicInstance | null) {
	if (element instanceof HTMLElement) {
		entryNameRefs.value[path] = element
	} else {
		delete entryNameRefs.value[path]
	}
}

function selectEntry(entry: FileTreeSelectEntry) {
	if (entry.type === 'directory') {
		navigateTo(entry.path)
	} else {
		toggleEntry(entry, !entry.checked)
	}
}

function toggleEntry(entry: FileTreeSelectEntry, selected: boolean) {
	if (entry.disabled) return

	const nextSelectedPaths = new Set(selectedPaths.value)
	const paths = getEntrySelectablePaths(entry)

	for (const path of paths) {
		if (selected) {
			nextSelectedPaths.add(path)
		} else {
			nextSelectedPaths.delete(path)
		}
	}

	emit('update:modelValue', [...nextSelectedPaths])
}

function getEntrySelectablePaths(entry: FileTreeSelectEntry) {
	if (entry.type === 'directory') {
		return entry.item?.type === 'directory'
			? [entry.path]
			: getFolderDescendants(entry.path)
					.filter((item) => !item.disabled)
					.map((item) => item.normalizedPath)
	}

	return [entry.path]
}

function toggleAllVisible(selected: boolean) {
	const nextSelectedPaths = new Set(selectedPaths.value)
	for (const path of visibleSelectablePaths.value) {
		if (selected) {
			nextSelectedPaths.add(path)
		} else {
			nextSelectedPaths.delete(path)
		}
	}

	emit('update:modelValue', [...nextSelectedPaths])
}

function formatSize(entry: FileTreeSelectEntry) {
	if (entry.type === 'directory') {
		return formatMessage(messages.itemCount, { count: entry.count ?? 0 })
	}

	if (entry.size === undefined) return ''
	return formatBytes(entry.size)
}

function formatModified(entry: FileTreeSelectEntry) {
	if (entry.modified === undefined) return ''
	return formatDateTime(new Date(entry.modified * 1000))
}
</script>
