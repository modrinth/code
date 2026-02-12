<script setup lang="ts">
import {
	BoxIcon,
	FilterIcon,
	GlassesIcon,
	PaintbrushIcon,
	PowerIcon,
	PowerOffIcon,
	SearchIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import { formatProjectType } from '@modrinth/utils'
import Fuse from 'fuse.js'
import { computed, nextTick, ref, watchSyncEffect } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils/common-messages'
import Avatar from '../../base/Avatar.vue'
import BulletDivider from '../../base/BulletDivider.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import Checkbox from '../../base/Checkbox.vue'
import FloatingActionBar from '../../base/FloatingActionBar.vue'
import StyledInput from '../../base/StyledInput.vue'
import NewModal from '../../modal/NewModal.vue'
import ContentCardTable from '../ContentCardTable.vue'
import type { ContentCardTableItem, ContentItem } from '../types'

const { formatMessage } = useVIntl()

interface Props {
	modpackName?: string
	modpackIconUrl?: string
	enableToggle?: boolean
}

const props = withDefaults(defineProps<Props>(), {
	modpackName: undefined,
	modpackIconUrl: undefined,
	enableToggle: false,
})

const emit = defineEmits<{
	'update:enabled': [item: ContentItem, value: boolean]
}>()

const messages = defineMessages({
	header: {
		id: 'instances.modpack-content-modal.header',
		defaultMessage: 'Modpack content',
	},
	searchPlaceholder: {
		id: 'instances.modpack-content-modal.search-placeholder',
		defaultMessage: 'Search {count} projects',
	},
	loading: {
		id: 'instances.modpack-content-modal.loading',
		defaultMessage: 'Loading content...',
	},
	emptyTitle: {
		id: 'instances.modpack-content-modal.empty-title',
		defaultMessage: 'No content found',
	},
	emptyDescription: {
		id: 'instances.modpack-content-modal.empty-description',
		defaultMessage: 'This modpack does not include any additional content.',
	},
	noResults: {
		id: 'instances.modpack-content-modal.no-results',
		defaultMessage: 'No projects match your search.',
	},
	backButton: {
		id: 'instances.modpack-content-modal.back-button',
		defaultMessage: 'Back',
	},
	allFilter: {
		id: 'instances.modpack-content-modal.filter-all',
		defaultMessage: 'All',
	},
	copyLink: {
		id: 'instances.modpack-content-modal.copy-link',
		defaultMessage: 'Copy link',
	},
	selectedCount: {
		id: 'instances.modpack-content-modal.selected-count',
		defaultMessage: '{count} selected',
	},
	enable: {
		id: 'instances.modpack-content-modal.enable',
		defaultMessage: 'Enable',
	},
	disable: {
		id: 'instances.modpack-content-modal.disable',
		defaultMessage: 'Disable',
	},
})

export interface ModpackContentModalState {
	items: ContentItem[]
	searchQuery: string
	selectedFilters: string[]
	scrollTop: number
}

const modal = ref<InstanceType<typeof NewModal>>()
const scrollContainer = ref<HTMLElement | null>(null)
const items = ref<ContentItem[]>([])
const loading = ref(false)
const searchQuery = ref('')
const selectedFilters = ref<string[]>([])
const selectedIds = ref<string[]>([])

const selectedItems = computed(() =>
	items.value.filter((item) => selectedIds.value.includes(item.file_name)),
)

const allSelected = computed(() => {
	if (filteredItems.value.length === 0) return false
	return filteredItems.value.every((item) => selectedIds.value.includes(item.file_name))
})

const someSelected = computed(() => {
	return (
		filteredItems.value.some((item) => selectedIds.value.includes(item.file_name)) &&
		!allSelected.value
	)
})

function toggleSelectAll() {
	if (allSelected.value || someSelected.value) {
		selectedIds.value = []
	} else {
		selectedIds.value = filteredItems.value.map((item) => item.file_name)
	}
}

const fuse = new Fuse<ContentItem>([], {
	keys: ['project.title', 'owner.name', 'file_name'],
	threshold: 0.4,
	distance: 100,
})

watchSyncEffect(() => fuse.setCollection(items.value))

const filterOptions = computed(() => {
	const frequency = items.value.reduce(
		(map, item) => {
			map[item.project_type] = (map[item.project_type] || 0) + 1
			return map
		},
		{} as Record<string, number>,
	)

	// Sort by frequency (most common first)
	return Object.entries(frequency)
		.sort(([, a], [, b]) => b - a)
		.map(([type]) => ({
			id: type,
			label: formatProjectType(type) + 's',
		}))
})

const stats = computed(() => {
	const counts: Record<string, number> = {}
	for (const item of items.value) {
		counts[item.project_type] = (counts[item.project_type] || 0) + 1
	}
	return counts
})

function toggleFilter(filterId: string) {
	const index = selectedFilters.value.indexOf(filterId)
	if (index === -1) {
		selectedFilters.value.push(filterId)
	} else {
		selectedFilters.value.splice(index, 1)
	}
}

const typeFilteredCount = computed(() => {
	if (selectedFilters.value.length === 0) return items.value.length
	return items.value.filter((item) => selectedFilters.value.includes(item.project_type)).length
})

const filteredItems = computed(() => {
	const query = searchQuery.value.trim()

	let result: ContentItem[]
	if (query) {
		result = fuse.search(query).map(({ item }) => item)
	} else {
		result = [...items.value].sort((a, b) => {
			const nameA = a.project?.title ?? a.file_name
			const nameB = b.project?.title ?? b.file_name
			return nameA.toLowerCase().localeCompare(nameB.toLowerCase())
		})
	}

	// Apply type filters
	if (selectedFilters.value.length > 0) {
		result = result.filter((item) => selectedFilters.value.includes(item.project_type))
	}

	return result
})

const tableItems = computed<ContentCardTableItem[]>(() =>
	filteredItems.value.map((item) => ({
		id: item.file_name,
		project: item.project ?? {
			id: item.file_name,
			slug: null,
			title: item.file_name,
			icon_url: null,
		},
		projectLink: item.project?.id ? `/project/${item.project.id}` : undefined,
		version: item.version ?? {
			id: item.file_name,
			version_number: 'Unknown',
			file_name: item.file_name,
		},
		owner: item.owner
			? {
					...item.owner,
					link: `https://modrinth.com/${item.owner.type}/${item.owner.id}`,
				}
			: undefined,
		...(props.enableToggle ? { enabled: item.enabled } : {}),
	})),
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

function handleEnabledChange(fileName: string, value: boolean) {
	const item = items.value.find((i) => i.file_name === fileName)
	if (!item) return
	emit('update:enabled', item, value)
}

function bulkEnable() {
	for (const item of selectedItems.value) {
		emit('update:enabled', item, true)
	}
	selectedIds.value = []
}

function bulkDisable() {
	for (const item of selectedItems.value) {
		emit('update:enabled', item, false)
	}
	selectedIds.value = []
}

function show(contentItems: ContentItem[]) {
	items.value = contentItems
	searchQuery.value = ''
	selectedFilters.value = []
	selectedIds.value = []
	loading.value = false
	modal.value?.show()
}

function showLoading() {
	items.value = []
	searchQuery.value = ''
	selectedFilters.value = []
	selectedIds.value = []
	loading.value = true
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function getState(): ModpackContentModalState | null {
	if (!items.value.length) return null
	return {
		items: items.value,
		searchQuery: searchQuery.value,
		selectedFilters: [...selectedFilters.value],
		scrollTop: scrollContainer.value?.scrollTop ?? 0,
	}
}

async function restore(state: ModpackContentModalState) {
	items.value = state.items
	searchQuery.value = state.searchQuery
	selectedFilters.value = state.selectedFilters
	loading.value = false
	modal.value?.show()
	await nextTick()
	if (scrollContainer.value) {
		scrollContainer.value.scrollTop = state.scrollTop
	}
}

defineExpose({ show, showLoading, hide, getState, restore })
</script>

<template>
	<NewModal
		ref="modal"
		:max-width="'min(928px, calc(95vw - 10rem))'"
		:width="'min(928px, calc(95vw - 10rem))'"
		no-padding
	>
		<template #title>
			<Avatar
				v-if="props.modpackIconUrl"
				:src="props.modpackIconUrl"
				size="3rem"
				:tint-by="props.modpackName"
			/>
			<span class="text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.header) }}
			</span>
		</template>
		<div class="flex flex-col h-[min(600px,calc(95vh-10rem))]">
			<div class="flex flex-col gap-4 px-6 py-4 border-b border-solid border-0 border-surface-4">
				<StyledInput
					v-model="searchQuery"
					:icon="SearchIcon"
					:placeholder="formatMessage(messages.searchPlaceholder, { count: typeFilteredCount })"
					clearable
				/>

				<!-- Filters -->
				<div v-if="filterOptions.length > 1" class="flex items-center gap-2">
					<FilterIcon class="size-5 text-secondary shrink-0" />
					<div class="flex flex-wrap items-center gap-1.5">
						<button
							class="rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-colors"
							:class="
								selectedFilters.length === 0
									? 'border-green bg-brand-highlight text-brand'
									: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
							"
							@click="selectedFilters = []"
						>
							{{ formatMessage(messages.allFilter) }}
						</button>
						<button
							v-for="option in filterOptions"
							:key="option.id"
							class="rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-colors"
							:class="
								selectedFilters.includes(option.id)
									? 'border-green bg-brand-highlight text-brand'
									: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
							"
							@click="toggleFilter(option.id)"
						>
							{{ option.label }}
						</button>
					</div>
				</div>
			</div>

			<!-- Content area -->
			<div class="flex-1 flex flex-col min-h-0 overflow-hidden">
				<!-- Loading state -->
				<div
					v-if="loading"
					class="flex flex-col items-center justify-center flex-1 gap-2 text-secondary"
				>
					<SpinnerIcon class="size-8 animate-spin" />
					<span class="text-sm">{{ formatMessage(messages.loading) }}</span>
				</div>

				<!-- Empty state -->
				<div
					v-else-if="items.length === 0"
					class="flex flex-col items-center justify-center flex-1 gap-2 text-center p-8"
				>
					<span class="text-xl font-semibold text-contrast">
						{{ formatMessage(messages.emptyTitle) }}
					</span>
					<span class="text-secondary">{{ formatMessage(messages.emptyDescription) }}</span>
				</div>

				<!-- No search results -->
				<div
					v-else-if="filteredItems.length === 0"
					class="flex flex-col items-center justify-center flex-1 gap-2 text-center p-8"
				>
					<span class="text-secondary">{{ formatMessage(messages.noResults) }}</span>
				</div>

				<!-- Content table -->
				<div v-else class="@container flex-1 min-h-0 flex flex-col">
					<div
						class="flex h-12 shrink-0 items-center justify-between gap-4 border-0 border-b border-solid border-surface-4 bg-surface-3 px-3"
					>
						<div
							class="flex min-w-0 items-center gap-4"
							:class="
								props.enableToggle
									? 'flex-1 @[800px]:w-[350px] @[800px]:shrink-0 @[800px]:flex-none'
									: 'flex-1'
							"
						>
							<Checkbox
								v-if="props.enableToggle"
								:model-value="allSelected"
								:indeterminate="someSelected"
								class="shrink-0"
								@update:model-value="toggleSelectAll"
							/>
							<span class="font-semibold text-secondary">{{
								formatMessage(commonMessages.projectLabel)
							}}</span>
						</div>
						<div
							class="hidden @[800px]:flex"
							:class="props.enableToggle ? 'w-[335px] min-w-0' : 'flex-1'"
						>
							<span class="font-semibold text-secondary">{{
								formatMessage(commonMessages.versionLabel)
							}}</span>
						</div>
						<div v-if="props.enableToggle" class="min-w-[160px] shrink-0 text-right">
							<span class="font-semibold text-secondary">{{
								formatMessage(commonMessages.actionsLabel)
							}}</span>
						</div>
					</div>
					<div ref="scrollContainer" class="flex-1 min-h-0 overflow-y-auto">
						<ContentCardTable
							v-model:selected-ids="selectedIds"
							:items="tableItems"
							:show-selection="props.enableToggle"
							hide-delete
							hide-header
							flat
							@update:enabled="(id, val) => handleEnabledChange(id, val)"
						/>
					</div>
				</div>
			</div>

			<!-- Footer -->
			<div
				class="flex items-center justify-between px-6 py-4 border-t border-solid border-0 border-surface-4 shrink-0"
			>
				<!-- Stats -->
				<div class="flex items-center gap-2">
					<template v-for="(count, type, idx) in stats" :key="type">
						<BulletDivider v-if="idx > 0" />
						<div class="flex items-center gap-1.5">
							<component :is="getTypeIcon(type as string)" class="size-5 text-secondary" />
							<span class="font-medium text-primary">
								{{ count }} {{ formatProjectType(type as string) }}{{ count !== 1 ? 's' : '' }}
							</span>
						</div>
					</template>
				</div>
			</div>
		</div>

		<FloatingActionBar
			v-if="props.enableToggle"
			:shown="selectedItems.length > 0"
			style="--left-bar-width: 0px; --right-bar-width: 0px"
		>
			<div class="flex items-center gap-0.5">
				<span class="px-4 py-2.5 text-base font-semibold text-contrast">
					{{ formatMessage(messages.selectedCount, { count: selectedItems.length }) }}
				</span>
				<div class="mx-1 h-6 w-px bg-surface-5" />
				<ButtonStyled type="transparent">
					<button class="!text-primary" @click="selectedIds = []">
						{{ formatMessage(commonMessages.clearButton) }}
					</button>
				</ButtonStyled>
			</div>

			<div class="ml-auto flex items-center gap-0.5">
				<ButtonStyled v-if="selectedItems.every((m) => !m.enabled)" type="transparent">
					<button @click="bulkEnable">
						<PowerIcon />
						{{ formatMessage(messages.enable) }}
					</button>
				</ButtonStyled>
				<ButtonStyled v-else type="transparent">
					<button @click="bulkDisable">
						<PowerOffIcon />
						{{ formatMessage(messages.disable) }}
					</button>
				</ButtonStyled>
			</div>
		</FloatingActionBar>
	</NewModal>
</template>
