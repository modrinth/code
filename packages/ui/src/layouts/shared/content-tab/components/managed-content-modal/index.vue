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
import { computed, nextTick, ref, watchSyncEffect } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import BulletDivider from '#ui/components/base/BulletDivider.vue'
import type { ButtonMenuOption } from '#ui/components/base/buttons'
import { ButtonLink } from '#ui/components/base/buttons'
import DropdownFilterBar from '#ui/components/base/DropdownFilterBar.vue'
import FilterPills from '#ui/components/base/FilterPills.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectPageContext } from '#ui/providers/page-context'
import {
	commonMessages,
	commonProjectTypeCategoryMessages,
	commonProjectTypeTitleMessages,
	normalizeProjectType,
} from '#ui/utils/common-messages'

import { getClientWarningType, getContentWarningType } from '../../composables/content-filtering'
import { type ContentSortMode, useContentSort } from '../../composables/content-sort'
import {
	type ContentMetadataFilterValue,
	useContentMetadataFilters,
} from '../../composables/use-content-metadata-filters'
import type {
	ContentCardProject,
	ContentCardTableItem,
	ContentItem,
	ContentSide,
} from '../../types'
import ContentCardTable from '../ContentCardTable.vue'
import ContentSelectionBar from '../ContentSelectionBar.vue'
import ContentSortMenu from '../ContentSortMenu.vue'

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
	filter: {
		id: 'content.page-layout.filter.add',
		defaultMessage: 'Filter',
	},
	warnings: {
		id: 'content.filter.warnings',
		defaultMessage: 'Warnings',
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
const { sortMode, sortedItems, hasDateAdded, resetSortMode } = useContentSort(items)
const { selectedMetadataFilters, metadataFilterCategories, applyMetadataFilters } =
	useContentMetadataFilters(items, undefined, {
		showEnabledFor: props.enableEnabledFor,
		showEnvironmentWarnings: props.showEnvironmentWarnings,
		showUpdates: false,
	})
const metadataFilterTriggerClass =
	'!h-[34px] !rounded-xl !border !border-solid !border-surface-5 !bg-transparent !px-3 !text-sm !font-medium !text-primary !shadow-[0_1px_1.5px_rgba(0,0,0,0.15)] transition-all duration-100 active:scale-[0.97] hover:!bg-surface-3 focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5'
const metadataFilterPreviewTriggerClass =
	'!h-[34px] !rounded-xl !border !border-solid !border-brand !bg-brand-highlight !px-3 !text-sm !font-medium !text-brand !shadow-[0_1px_1.5px_rgba(0,0,0,0.15)] transition-all duration-100 active:scale-[0.97] hover:!bg-brand-highlight focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5 [&>svg]:!text-brand'
const selectedIds = ref<string[]>([])
const highlightedItemId = ref<string>()

function updateFilters(filters: string[]) {
	selectedFilters.value = props.filterMode === 'status' ? filters.slice(-1) : filters
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

const filterOptions = computed(() => {
	if (props.filterMode === 'status') {
		return [
			{ id: 'enabled', label: formatMessage(messages.enabled) },
			{ id: 'disabled', label: formatMessage(messages.disabled) },
		]
	}

	const frequency = items.value.reduce(
		(map, item) => {
			const normalized = normalizeProjectType(item.project_type)
			map[normalized] = (map[normalized] || 0) + 1
			return map
		},
		{} as Record<string, number>,
	)

	const options = Object.entries(frequency)
		.sort(([, a], [, b]) => b - a)
		.map(([type]) => {
			const msg =
				commonProjectTypeCategoryMessages[type as keyof typeof commonProjectTypeCategoryMessages]
			return {
				id: type,
				label: msg ? formatMessage(msg) : type.charAt(0).toUpperCase() + type.slice(1) + 's',
			}
		})

	if (
		applyMetadataFilters(items.value).some(
			(item) => getContentWarningType(item, props.showEnvironmentWarnings) !== null,
		)
	) {
		options.push({ id: 'warnings', label: formatMessage(messages.warnings) })
	}

	return options
})

watchSyncEffect(() => {
	if (items.value.length === 0) return
	const availableFilters = new Set(filterOptions.value.map((option) => option.id))
	const validFilters = selectedFilters.value.filter((filter) => availableFilters.has(filter))
	if (validFilters.length !== selectedFilters.value.length) selectedFilters.value = validFilters
})

const attributeFilterIds = new Set(['enabled', 'disabled', 'warnings'])

function matchesSelectedFilters(item: ContentItem) {
	const typeFilters = selectedFilters.value.filter((f) => !attributeFilterIds.has(f))
	const hasEnabledFilter = props.enableToggle && selectedFilters.value.includes('enabled')
	const hasDisabledFilter = props.enableToggle && selectedFilters.value.includes('disabled')
	const hasWarningsFilter = selectedFilters.value.includes('warnings')
	if (typeFilters.length > 0 && !typeFilters.includes(normalizeProjectType(item.project_type)))
		return false
	if (hasEnabledFilter !== hasDisabledFilter && Boolean(item.enabled) !== hasEnabledFilter)
		return false
	if (hasWarningsFilter && getContentWarningType(item, props.showEnvironmentWarnings) === null)
		return false
	return true
}

const searchableCount = computed(
	() => applyMetadataFilters(items.value.filter(matchesSelectedFilters)).length,
)

const filteredItems = computed(() => {
	const query = searchQuery.value.trim()

	let result = query ? fuse.value.search(query).map(({ item }) => item) : sortedItems.value

	if (selectedFilters.value.length > 0) {
		result = result.filter(matchesSelectedFilters)
	}

	return applyMetadataFilters(result)
})

const stats = computed(() => {
	const counts: Record<string, number> = {}
	for (const item of filteredItems.value) {
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
		isClientOnly:
			!props.enableEnabledFor && getClientWarningType(item, props.showEnvironmentWarnings) !== null,
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
	() => props.enableToggle || props.enableEnabledFor || hasExternalSlicerUrls.value,
)

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

function show(contentItems: ContentItem[], highlightId?: string) {
	items.value = contentItems.map((item) => ({ ...item }))
	highlightedItemId.value = highlightId
	searchQuery.value = ''
	selectedFilters.value = []
	selectedMetadataFilters.value = {}
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
	selectedMetadataFilters.value = {}
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
	if (!items.value.length) return null
	return {
		items: items.value,
		searchQuery: searchQuery.value,
		selectedFilters: [...selectedFilters.value],
		selectedMetadataFilters: { ...selectedMetadataFilters.value },
		sortMode: sortMode.value,
		scrollTop: scrollContainer.value?.scrollTop ?? 0,
	}
}

async function restore(state: ManagedContentModalState) {
	items.value = state.items.map((item) => ({ ...item }))
	highlightedItemId.value = undefined
	searchQuery.value = state.searchQuery
	selectedFilters.value = state.selectedFilters
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

				<div class="flex flex-wrap items-center gap-2">
					<ContentSortMenu v-model="sortMode" :show-date-options="hasDateAdded" />
					<template v-if="filterOptions.length > 0">
						<div class="h-6 w-px shrink-0 bg-surface-5" />
						<FilterPills
							:model-value="selectedFilters"
							:options="filterOptions"
							@update:model-value="updateFilters"
						>
							<template #all>
								{{ formatMessage(commonMessages.allProjectType) }}
							</template>
						</FilterPills>
					</template>
					<div
						v-if="metadataFilterCategories.length > 0"
						class="flex flex-wrap items-center gap-1.5 [&>div:last-of-type]:!h-[34px] [&>div:last-of-type]:!gap-1.5 [&_[data-button]]:!h-[34px]"
					>
						<DropdownFilterBar
							v-model="selectedMetadataFilters"
							:categories="metadataFilterCategories"
							:show-label="false"
							:add-label="formatMessage(messages.filter)"
							:add-button-class="metadataFilterTriggerClass"
							:preview-trigger-class="metadataFilterPreviewTriggerClass"
							add-button-size="sm"
							checkbox-position="right"
							apply-immediately
						>
							<template #preview-content="{ label, summary }">
								<span class="min-w-0 flex-1 truncate">
									<span class="font-medium">{{ label }}:</span>
									<span class="ml-1 font-semibold text-contrast">{{ summary }}</span>
								</span>
							</template>
							<template #option="{ option, selected }">
								<span
									class="min-w-0 truncate font-semibold leading-tight"
									:class="selected ? 'text-contrast' : 'text-primary'"
								>
									{{ option.label }}
								</span>
							</template>
						</DropdownFilterBar>
					</div>
				</div>
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
						<ContentCardTable
							v-model:selected-ids="selectedIds"
							:items="tableItems"
							:highlighted-item-id="highlightedItemId"
							:show-selection="props.enableToggle"
							:show-item-actions="showTableActions"
							:get-additional-action-widths="
								(item) => [
									...(externalSlicerUrls[item.id] ? [36] : []),
									...(externalUrls[item.id] ? [36] : []),
								]
							"
							:show-version="showVersion"
							:show-enabled-for-column="props.enableEnabledFor"
							hide-delete
							flat
							v-on="
								props.enableToggle || props.enableEnabledFor
									? { 'update:enabled': (id: string, val: boolean) => handleEnabledChange(id, val) }
									: {}
							"
							@update:enabled-for="handleEnabledForChange"
						>
							<template #itemButtonsRight="{ item }">
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
						</ContentCardTable>
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
