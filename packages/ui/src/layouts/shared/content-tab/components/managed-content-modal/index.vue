<script setup lang="ts">
import {
	ArrowLeftRightIcon,
	BoxIcon,
	ExternalIcon,
	FileIcon,
	GlassesIcon,
	PaintbrushIcon,
	SearchIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import Fuse from 'fuse.js'
import { computed, nextTick, ref, watch } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import BulletDivider from '#ui/components/base/BulletDivider.vue'
import type { ButtonMenuOption } from '#ui/components/base/buttons'
import { ButtonLink } from '#ui/components/base/buttons'
import FilterPills from '#ui/components/base/FilterPills.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectPageContext } from '#ui/providers/page-context'
import {
	commonMessages,
	commonProjectTypeTitleMessages,
	normalizeProjectType,
} from '#ui/utils/common-messages'

import { getClientWarningType } from '../../composables/content-filtering'
import { type ContentSortMode, useContentSort } from '../../composables/content-sort'
import {
	type ContentMetadataFilterValue,
	useContentMetadataFilters,
} from '../../composables/use-content-metadata-filters'
import { contentTypeOf, contentTypeOrder } from '../../composables/use-content-view-state'
import type {
	ContentCardProject,
	ContentCardTableItem,
	ContentItem,
	ContentSide,
	ContentType,
} from '../../types'
import ContentTable from '../content-table/index.vue'
import type { ContentTableColumns } from '../content-table/types'
import ContentSelectionBar from '../ContentSelectionBar.vue'
import FilterRow from '../filter-row.vue'

const { formatMessage } = useVIntl()
const pageContext = injectPageContext(null)

interface Props {
	description?: string
	emptyDescription?: string
	header?: string
	sourceName?: string
	sourceIconUrl?: string
	enableToggle?: boolean
	enableEnabledFor?: boolean
	actionDisabled?: boolean
	actionDisabledTooltip?: string | null
	getOverflowOptions?: (item: ContentItem) => ButtonMenuOption[]
	switchVersion?: (item: ContentItem) => void
	showVersion?: boolean
	showEnvironmentWarnings?: boolean
	filterMode?: 'content' | 'status'
	/** Treats files without a Modrinth project as modpack overrides. */
	overrideFiles?: boolean
}

const props = withDefaults(defineProps<Props>(), {
	header: undefined,
	description: undefined,
	emptyDescription: undefined,
	sourceName: undefined,
	sourceIconUrl: undefined,
	enableToggle: false,
	enableEnabledFor: false,
	actionDisabled: false,
	actionDisabledTooltip: undefined,
	getOverflowOptions: undefined,
	switchVersion: undefined,
	showVersion: true,
	showEnvironmentWarnings: false,
	filterMode: 'content',
	overrideFiles: false,
})

const emit = defineEmits<{
	'update:enabled': [item: ContentItem, value: boolean]
	'update:enabled-for': [item: ContentItem, side: ContentSide, value: boolean]
	'bulk:enable': [items: ContentItem[]]
	'bulk:disable': [items: ContentItem[]]
	hide: []
}>()

const messages = defineMessages({
	header: {
		id: 'instances.managed-content-modal.header',
		defaultMessage: 'Managed content',
	},
	searchPlaceholder: {
		id: 'instances.managed-content-modal.search-placeholder',
		defaultMessage: 'Search {count, number} {count, plural, one {project} other {projects}}',
	},
	loading: {
		id: 'instances.managed-content-modal.loading',
		defaultMessage: 'Loading content...',
	},
	emptyTitle: {
		id: 'instances.managed-content-modal.empty-title',
		defaultMessage: 'No content found',
	},
	emptyDescription: {
		id: 'instances.managed-content-modal.empty-description',
		defaultMessage: 'This source does not include any managed content.',
	},
	noResults: {
		id: 'instances.managed-content-modal.no-results',
		defaultMessage: 'No projects match your search.',
	},
	openInSlicer: {
		id: 'instances.managed-content-modal.open-in-slicer',
		defaultMessage: 'Open in Slicer',
	},
	downloadFile: {
		id: 'instances.managed-content-modal.download-file',
		defaultMessage: 'Download File',
	},
	enabled: {
		id: 'instances.managed-content-modal.enabled',
		defaultMessage: 'Enabled',
	},
	disabled: {
		id: 'instances.managed-content-modal.disabled',
		defaultMessage: 'Disabled',
	},
	pleaseWait: {
		id: 'content.enabled-for.please-wait',
		defaultMessage: 'Please wait',
	},
})

export interface ManagedContentModalState {
	items: ContentItem[]
	searchQuery: string
	selectedFilters: string[]
	selectedMetadataFilters?: ContentMetadataFilterValue
	activeType?: ContentType
	sortMode?: ContentSortMode
	scrollTop: number
}

const modal = ref<InstanceType<typeof NewModal>>()
const scrollContainer = ref<HTMLElement | null>(null)
const isOpen = ref(false)
const items = ref<ContentItem[]>([])
const disabledIds = ref(new Set<string>())
const loading = ref(false)
const searchQuery = ref('')
const selectedFilters = ref<string[]>([])
const { sortMode, sortedItems, sortFields, resetSortMode } = useContentSort(items)
const showTypeTabs = computed(() => props.filterMode === 'content')

const types = computed<ContentType[]>(() => {
	const present = new Set(items.value.map(contentTypeOf))
	return [
		...contentTypeOrder.filter((type) => present.has(type)),
		...[...present].filter((type) => !contentTypeOrder.includes(type)),
	]
})
const activeType = ref<ContentType>('mod')
watch(types, (next) => {
	if (next.length > 0 && !next.includes(activeType.value)) activeType.value = next[0]
})

function inActiveType(item: ContentItem) {
	return !showTypeTabs.value || contentTypeOf(item) === activeType.value
}

const typeItems = computed(() => items.value.filter(inActiveType))

const metadataFiltersByType = ref<Partial<Record<ContentType, ContentMetadataFilterValue>>>({})
const selectedMetadataFilters = computed<ContentMetadataFilterValue>({
	get: () => metadataFiltersByType.value[activeType.value] ?? {},
	set: (value) => {
		metadataFiltersByType.value = { ...metadataFiltersByType.value, [activeType.value]: value }
	},
})
const { metadataFilterCategories, applyMetadataFilters } = useContentMetadataFilters(
	typeItems,
	selectedMetadataFilters,
	{
		showEnabledFor: props.enableEnabledFor,
		showEnvironmentWarnings: props.showEnvironmentWarnings,
		showUpdates: false,
	},
)
const selectedIds = ref<string[]>([])
const highlightedItemId = ref<string>()

function updateFilters(filters: string[]) {
	selectedFilters.value = filters.slice(-1)
}

const selectedItems = computed(() =>
	items.value.filter((item) => selectedIds.value.includes(item.id)),
)
const toggleableSelectedItems = computed(() => selectedItems.value)

const fuse = computed(
	() =>
		new Fuse<ContentItem>(items.value, {
			keys: ['project.title', 'owner.name', 'file_name'],
			threshold: 0.4,
			distance: 100,
		}),
)

const statusFilterOptions = computed(() => [
	{ id: 'enabled', label: formatMessage(messages.enabled) },
	{ id: 'disabled', label: formatMessage(messages.disabled) },
])

function matchesSelectedFilters(item: ContentItem) {
	if (!inActiveType(item)) return false
	const hasEnabledFilter = props.enableToggle && selectedFilters.value.includes('enabled')
	const hasDisabledFilter = props.enableToggle && selectedFilters.value.includes('disabled')
	return hasEnabledFilter === hasDisabledFilter || Boolean(item.enabled) === hasEnabledFilter
}

const searchableCount = computed(
	() => applyMetadataFilters(items.value.filter(matchesSelectedFilters)).length,
)

const searchedItems = computed(() => {
	const query = searchQuery.value.trim()
	return query ? fuse.value.search(query).map(({ item }) => item) : sortedItems.value
})

const filteredItems = computed(() =>
	applyMetadataFilters(searchedItems.value.filter(matchesSelectedFilters)),
)

const stats = computed(() => {
	const counts: Record<string, number> = {}
	for (const item of searchedItems.value) {
		const normalized = normalizeProjectType(item.project_type)
		counts[normalized] = (counts[normalized] || 0) + 1
	}
	return counts
})

function isExternalItem(item: ContentItem) {
	return item.external ?? (props.overrideFiles && !item.project)
}

const tableItems = computed<ContentCardTableItem[]>(() =>
	filteredItems.value.map((item) => ({
		id: item.id,
		projectType: item.project_type,
		project: item.project ?? {
			id: item.id,
			slug: null,
			title: item.embedded_metadata?.name ?? item.file_name,
			icon_url: item.embedded_metadata?.icon_url ?? null,
		},
		projectLink:
			!isExternalItem(item) && item.project?.id ? `/project/${item.project.id}` : undefined,
		embeddedIcon: item.embeddedIcon,
		version: props.showVersion
			? (item.version ?? {
					id: item.id,
					version_number: item.embedded_metadata?.version ?? '',
					file_name: item.file_name,
				})
			: undefined,
		owner: item.owner
			? {
					...item.owner,
					link:
						item.owner.type === 'user'
							? `/user/${encodeURIComponent(item.owner.id)}`
							: `https://modrinth.com/organization/${item.owner.id}`,
				}
			: undefined,
		source: item.source
			? {
					...item.source,
					link: item.source.link ?? sourceProjectLink(item.source.project),
				}
			: undefined,
		external: isExternalItem(item),
		externalFile:
			item.external &&
			(!!item.external_url ||
				item.source_kind === 'modrinth_modpack' ||
				item.source_kind === 'imported_modpack'),
		overrideFile: props.overrideFiles && isExternalItem(item) && !item.external_url,
		...(props.enableToggle || props.enableEnabledFor ? { enabled: item.enabled } : {}),
		...(props.enableEnabledFor ? { enabledFor: item.enabledFor } : {}),
		synced: !!item.synced_pack,
		syncUpdatePending: item.synced_pack?.update_pending,
		locked: item.locked,
		installing: item.installing === true,
		toggleDisabled: props.actionDisabled,
		toggleDisabledTooltip: props.actionDisabled ? props.actionDisabledTooltip : undefined,
		clientWarning: props.enableEnabledFor
			? null
			: getClientWarningType(item, props.showEnvironmentWarnings),
		disabled:
			props.actionDisabled || disabledIds.value.has(item.file_name) || item.installing === true,
		disabledTooltip: props.actionDisabled
			? props.actionDisabledTooltip
			: disabledIds.value.has(item.file_name)
				? formatMessage(messages.pleaseWait)
				: undefined,
		overflowOptions: [
			...(props.switchVersion && !item.locked && item.project?.id && item.version?.id
				? [
						{
							id: 'switch-version',
							label: formatMessage(commonMessages.switchVersionButton),
							icon: ArrowLeftRightIcon,
							action: () => props.switchVersion!(item),
						},
					]
				: []),
			...(props.getOverflowOptions?.(item) ?? []),
		],
	})),
)
const externalSlicerUrls = computed(() => {
	const urls: Record<string, string> = {}
	for (const item of items.value) {
		if (item.external && item.external_url) {
			urls[item.id] = `https://slicer.run/?url=${encodeURIComponent(item.external_url)}`
		}
	}
	return urls
})
const externalUrls = computed(() => {
	const urls: Record<string, string> = {}
	for (const item of items.value) {
		if (item.external && item.external_url) {
			urls[item.id] = item.external_url
		}
	}
	return urls
})
const hasExternalSlicerUrls = computed(() => Object.keys(externalSlicerUrls.value).length > 0)
const showTableActions = computed(
	() =>
		props.enableToggle ||
		props.enableEnabledFor ||
		hasExternalSlicerUrls.value ||
		tableItems.value.some((item) => item.overflowOptions?.length || item.synced || item.locked),
)
const tableColumns = computed<ContentTableColumns>(() => ({
	selection: props.enableToggle,
	version: props.showVersion,
	enabledFor: props.enableEnabledFor,
	actions: showTableActions.value
		? props.enableToggle || props.enableEnabledFor
			? ['toggle']
			: []
		: undefined,
}))

function getTypeIcon(type: string) {
	switch (type) {
		case 'mod':
			return BoxIcon
		case 'shaderpack':
		case 'shader':
			return GlassesIcon
		case 'resourcepack':
			return PaintbrushIcon
		default:
			return BoxIcon
	}
}

function sourceProjectLink(project: ContentCardProject) {
	const projectId = project.slug ?? project.id
	const url = `https://modrinth.com/modpack/${encodeURIComponent(projectId)}`
	return pageContext ? () => pageContext.openExternalUrl(url) : url
}

function handleEnabledChange(id: string, value: boolean) {
	if (props.actionDisabled) return
	const item = items.value.find((item) => item.id === id)
	if (!item) return
	emit('update:enabled', item, value)
}

function handleEnabledForChange(id: string, side: ContentSide, value: boolean) {
	if (props.actionDisabled || !props.enableEnabledFor) return
	const item = items.value.find((item) => item.id === id)
	if (!item) return
	emit('update:enabled-for', item, side, value)
}

function bulkEnable() {
	if (props.actionDisabled) return
	emit('bulk:enable', [...toggleableSelectedItems.value])
	selectedIds.value = []
}

function bulkDisable() {
	if (props.actionDisabled) return
	emit('bulk:disable', [...toggleableSelectedItems.value])
	selectedIds.value = []
}

function resetTypeFilters(highlightId?: string) {
	const highlighted = highlightId ? items.value.find((item) => item.id === highlightId) : undefined
	activeType.value = highlighted ? contentTypeOf(highlighted) : (types.value[0] ?? 'mod')
	metadataFiltersByType.value = {}
}

function show(contentItems: ContentItem[], highlightId?: string) {
	items.value = contentItems.map((item) => ({ ...item }))
	highlightedItemId.value = highlightId
	searchQuery.value = ''
	selectedFilters.value = []
	resetTypeFilters(highlightId)
	resetSortMode()
	selectedIds.value = []
	disabledIds.value = new Set()
	loading.value = false
	showModal()
}

function showLoading() {
	items.value = []
	highlightedItemId.value = undefined
	searchQuery.value = ''
	selectedFilters.value = []
	resetTypeFilters()
	resetSortMode()
	selectedIds.value = []
	loading.value = true
	showModal()
}

function showModal() {
	if (isOpen.value || !modal.value) return
	isOpen.value = true
	modal.value.show()
}

function hide() {
	isOpen.value = false
	modal.value?.hide()
}

function handleHide() {
	isOpen.value = false
	emit('hide')
}

function getState(): ManagedContentModalState | null {
	if (!isOpen.value || !items.value.length) return null
	return {
		items: items.value,
		searchQuery: searchQuery.value,
		selectedFilters: [...selectedFilters.value],
		selectedMetadataFilters: { ...selectedMetadataFilters.value },
		activeType: activeType.value,
		sortMode: sortMode.value,
		scrollTop: scrollContainer.value?.scrollTop ?? 0,
	}
}

async function restore(state: ManagedContentModalState) {
	items.value = state.items.map((item) => ({ ...item }))
	highlightedItemId.value = undefined
	searchQuery.value = state.searchQuery
	selectedFilters.value = state.selectedFilters
	resetTypeFilters()
	if (state.activeType && types.value.includes(state.activeType)) {
		activeType.value = state.activeType
	}
	selectedMetadataFilters.value = state.selectedMetadataFilters ?? {}
	if (state.sortMode) sortMode.value = state.sortMode
	else resetSortMode()
	loading.value = false
	showModal()
	await nextTick()
	if (scrollContainer.value) {
		scrollContainer.value.scrollTop = state.scrollTop
	}
}

function updateItem(fileName: string, updates: Partial<ContentItem> & { disabled?: boolean }) {
	if (updates.disabled !== undefined) {
		const newSet = new Set(disabledIds.value)
		if (updates.disabled) {
			newSet.add(fileName)
		} else {
			newSet.delete(fileName)
		}
		disabledIds.value = newSet
	}
	const { disabled: _, ...itemUpdates } = updates
	if (Object.keys(itemUpdates).length > 0) {
		items.value = items.value.map((item) =>
			item.file_name === fileName ? { ...item, ...itemUpdates } : item,
		)
	}
}

function setItems(contentItems: ContentItem[]) {
	const contentIds = new Set(contentItems.map((item) => item.id))
	const contentFileNames = new Set(contentItems.map((item) => item.file_name))
	items.value = contentItems.map((item) => ({ ...item }))
	selectedIds.value = selectedIds.value.filter((id) => contentIds.has(id))
	disabledIds.value = new Set([...disabledIds.value].filter((id) => contentFileNames.has(id)))
	loading.value = false
}

defineExpose({ show, showLoading, hide, getState, restore, updateItem, setItems })
</script>

<template>
	<NewModal
		ref="modal"
		:max-width="
			props.enableEnabledFor ? 'min(1080px, calc(95vw - 4rem))' : 'min(928px, calc(95vw - 10rem))'
		"
		:width="
			props.enableEnabledFor ? 'min(1080px, calc(95vw - 4rem))' : 'min(928px, calc(95vw - 10rem))'
		"
		:on-hide="handleHide"
		no-padding
	>
		<template #title>
			<Avatar
				v-if="props.sourceIconUrl"
				:src="props.sourceIconUrl"
				cache-icon
				:alt="props.sourceName"
				size="3rem"
				:tint-by="props.sourceName"
			/>
			<span class="text-2xl font-semibold text-contrast">
				{{ props.header ?? formatMessage(messages.header) }}
			</span>
		</template>
		<div class="flex max-h-[min(600px,calc(95vh-10rem))] flex-col">
			<div
				class="flex shrink-0 flex-col gap-4 px-6 py-4 border-b border-solid border-0 border-surface-4"
			>
				<p v-if="description" class="m-0 text-secondary">{{ description }}</p>
				<slot name="toolbar" />
				<Input
					v-model="searchQuery"
					:icon="SearchIcon"
					type="search"
					:placeholder="formatMessage(messages.searchPlaceholder, { count: searchableCount })"
					:aria-label="formatMessage(messages.searchPlaceholder, { count: searchableCount })"
					:clear-label="formatMessage(commonMessages.clearButton)"
					clearable
				/>

				<FilterRow
					v-model:type="activeType"
					v-model:filters="selectedMetadataFilters"
					v-model:sort="sortMode"
					:types="types"
					:filter-categories="metadataFilterCategories"
					:sort-fields="sortFields"
					flat
				>
					<template v-if="filterMode === 'status'" #leading>
						<FilterPills
							:model-value="selectedFilters"
							:options="statusFilterOptions"
							@update:model-value="updateFilters"
						>
							<template #all>
								{{ formatMessage(commonMessages.allProjectType) }}
							</template>
						</FilterPills>
					</template>
				</FilterRow>
			</div>

			<div class="flex min-h-0 flex-col overflow-hidden">
				<div
					v-if="loading"
					class="flex flex-col items-center justify-center gap-2 p-8 text-secondary"
				>
					<SpinnerIcon class="size-8 animate-spin" />
					<span class="text-sm">{{ formatMessage(messages.loading) }}</span>
				</div>

				<div
					v-else-if="items.length === 0"
					class="flex flex-col items-center justify-center flex-1 gap-2 text-center p-8"
				>
					<span class="text-xl font-semibold text-contrast">
						{{ formatMessage(messages.emptyTitle) }}
					</span>
					<span class="text-secondary">{{
						emptyDescription ?? formatMessage(messages.emptyDescription)
					}}</span>
				</div>

				<div
					v-else-if="filteredItems.length === 0"
					class="flex flex-col items-center justify-center flex-1 gap-2 text-center p-8"
				>
					<span class="text-secondary">{{ formatMessage(messages.noResults) }}</span>
				</div>

				<div v-else class="@container flex min-h-0 flex-col">
					<div ref="scrollContainer" class="min-h-0 overflow-y-auto">
						<ContentTable
							v-model:selected-ids="selectedIds"
							:sections="[{ id: 'managed', items: tableItems }]"
							:columns="tableColumns"
							:highlighted-item-id="highlightedItemId"
							:extra-action-widths="
								(item) => [
									...(externalSlicerUrls[item.id] ? [36] : []),
									...(externalUrls[item.id] ? [36] : []),
								]
							"
							flat
							@toggle="handleEnabledChange"
							@enabled-for="handleEnabledForChange"
						>
							<template #row-actions-after="{ item }">
								<ButtonLink
									v-if="externalSlicerUrls[item.id]"
									v-tooltip="formatMessage(messages.openInSlicer)"
									type="quiet"
									:href="externalSlicerUrls[item.id]"
									target="_blank"
									rel="noopener noreferrer"
									class="!w-9 !px-0 !rounded-full"
								>
									<ExternalIcon class="size-4" />
								</ButtonLink>
								<ButtonLink
									v-if="externalUrls[item.id]"
									v-tooltip="formatMessage(messages.downloadFile)"
									type="quiet"
									:href="externalUrls[item.id]"
									target="_blank"
									rel="noopener noreferrer"
									class="!w-9 !px-0 !rounded-full"
								>
									<FileIcon class="size-4" />
								</ButtonLink>
							</template>
						</ContentTable>
					</div>
				</div>
			</div>

			<div
				class="flex items-center justify-between px-6 py-4 border-t border-solid border-0 border-surface-4 shrink-0"
			>
				<div class="flex items-center gap-2">
					<template v-for="(count, type, idx) in stats" :key="type">
						<BulletDivider v-if="idx > 0" />
						<div class="flex items-center gap-1.5">
							<component :is="getTypeIcon(type as string)" class="size-5 text-secondary" />
							<span class="font-medium text-primary">
								{{ count }}
								{{
									formatMessage(
										commonProjectTypeTitleMessages[
											normalizeProjectType(
												type as string,
											) as keyof typeof commonProjectTypeTitleMessages
										] ?? commonProjectTypeTitleMessages.project,
										{ count },
									)
								}}
							</span>
						</div>
					</template>
				</div>
			</div>
		</div>

		<ContentSelectionBar
			v-if="props.enableToggle"
			:selected-items="selectedItems"
			:toggle-items="toggleableSelectedItems"
			:is-busy="props.actionDisabled"
			:busy-tooltip="props.actionDisabledTooltip"
			:hide-when-modal-open="false"
			style="--left-bar-width: 0px; --right-bar-width: 0px"
			@clear="selectedIds = []"
			@enable="bulkEnable"
			@disable="bulkDisable"
		/>
	</NewModal>
</template>
