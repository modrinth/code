<script setup lang="ts">
import {
	CodeIcon,
	DownloadIcon,
	DropdownIcon,
	FileIcon,
	LinkIcon,
	ShareIcon,
	TextCursorInputIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed, ref, watch } from 'vue'

import { Button, TeleportOverflowMenu } from '#ui/components/base/buttons'
import UpdateAllModal from '#ui/components/modal/update-all-modal/index.vue'
import { useDebugLogger } from '#ui/composables/debug-logger'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	commonMessages,
	formatContentTypeSentence,
	normalizeProjectType,
} from '#ui/utils/common-messages'

import ContentTable from './components/content-table/index.vue'
import ContentSelectionBar from './components/ContentSelectionBar.vue'
import ContentEmptyState from './components/empty-state.vue'
import ManagedContentCard from './components/managed-content-card/index.vue'
import ConfirmDeletionModal from './components/modals/ConfirmDeletionModal.vue'
import ConfirmDisableModal from './components/modals/ConfirmDisableModal.vue'
import ConfirmUnlinkModal from './components/modals/ConfirmUnlinkModal.vue'
import ContentDependencyWarningModal from './components/modals/ContentDependencyWarningModal.vue'
import ContentToolbar from './components/toolbar/index.vue'
import {
	getClientWarningType,
	useBulkOperation,
	useChangingItems,
	useContentDeleteFlow,
	useContentMetadataFilters,
	useContentSearch,
	useContentSelection,
	useContentSort,
	useContentToggleFlow,
	useContentUpdateAllFlow,
} from './composables'
import { injectContentManager } from './providers/content-manager'
import type { ContentCardTableItem, ContentItem, ContentType } from './types'

const { formatMessage } = useVIntl()
const debug = useDebugLogger('ContentPageLayout')

const props = withDefaults(
	defineProps<{
		bottomPadding?: boolean
		highlightedItemId?: string
	}>(),
	{
		bottomPadding: true,
	},
)

const messages = defineMessages({
	loadingContent: {
		id: 'content.page-layout.loading',
		defaultMessage: 'Loading content...',
	},
	failedToLoad: {
		id: 'content.page-layout.failed-to-load',
		defaultMessage: 'Failed to load content',
	},
	additionalContent: {
		id: 'content.page-layout.additional-content',
		defaultMessage: 'Additional content',
	},
	searchPlaceholder: {
		id: 'content.page-layout.search-placeholder',
		defaultMessage: 'Search {count, number} {contentType}...',
	},
	noContentFound: {
		id: 'content.page-layout.no-content-found',
		defaultMessage: 'No content found.',
	},
	noExtraContentInstalled: {
		id: 'content.page-layout.empty.no-extra-content-installed',
		defaultMessage: 'No extra content installed',
	},
	noContentInstalled: {
		id: 'content.page-layout.empty.no-content-installed',
		defaultMessage: 'No content installed',
	},
	emptyModpackHint: {
		id: 'content.page-layout.empty.modpack-hint',
		defaultMessage: 'Add additional content on top of this modpack',
	},
	emptyHint: {
		id: 'content.page-layout.empty.projects-hint',
		defaultMessage: 'Browse or upload projects to get started.',
	},
	noTypeInstalled: {
		id: 'content.page-layout.empty.no-type-installed',
		defaultMessage: 'No {contentType} installed',
	},
	shareProjectNames: {
		id: 'content.page-layout.share.project-names',
		defaultMessage: 'Project names',
	},
	shareFileNames: {
		id: 'content.page-layout.share.file-names',
		defaultMessage: 'File names',
	},
	shareProjectLinks: {
		id: 'content.page-layout.share.project-links',
		defaultMessage: 'Project links',
	},
	shareMarkdownLinks: {
		id: 'content.page-layout.share.markdown-links',
		defaultMessage: 'Markdown links',
	},
	share: {
		id: 'content.page-layout.share.label',
		defaultMessage: 'Share',
	},
	pleaseWait: {
		id: 'content.page-layout.please-wait',
		defaultMessage: 'Please wait',
	},
})

const ctx = injectContentManager()

function getItemId(item: ContentItem) {
	return ctx.getItemId?.(item) ?? item.file_path ?? item.file_name ?? item.id
}

const contentTypeOrder: ContentType[] = ['mod', 'plugin', 'resourcepack', 'datapack', 'shader']

const types = computed<ContentType[]>(() => {
	const provided = ctx.types.value
	const present = new Set(ctx.items.value.map((item) => normalizeProjectType(item.project_type)))
	return [
		...provided,
		...contentTypeOrder.filter((type) => present.has(type) && !provided.includes(type)),
	]
})
const activeType = ref<ContentType>(types.value[0] ?? 'mod')
watch(types, (next) => {
	if (!next.includes(activeType.value)) activeType.value = next[0] ?? 'mod'
})
const typeItems = computed(() =>
	ctx.items.value.filter((item) => normalizeProjectType(item.project_type) === activeType.value),
)

const isServer = computed(() => ctx.target.value.kind === 'server')
const addTooltip = computed(
	() => ctx.busy.value?.message ?? (ctx.canAdd.value ? undefined : ctx.addDeniedReason),
)

const { sortMode, sortedItems, hasDateAdded } = useContentSort(typeItems, ctx.persistKey)

const { searchQuery, search } = useContentSearch(
	sortedItems,
	['project.title', 'owner.name', 'file_name'],
	{ queryParam: ctx.searchQueryParam },
)

const retainedRowIds = ref(new Set<string>())

const { selectedMetadataFilters, metadataFilterCategories, applyMetadataFilters } =
	useContentMetadataFilters(typeItems, ctx.persistKey, {
		showSharedContent: ctx.showSharedContentFilter,
		showEnabledFor: !!ctx.setEnabledFor,
		showEnvironmentWarnings: isServer.value,
	})

watch([searchQuery, selectedMetadataFilters, activeType], () => retainedRowIds.value.clear())

watch(
	() => props.highlightedItemId,
	(id) => {
		if (!id) return
		searchQuery.value = ''
		selectedMetadataFilters.value = {}
		const item = ctx.items.value.find((candidate) => getItemId(candidate) === id)
		if (item) activeType.value = normalizeProjectType(item.project_type) as ContentType
	},
	{ immediate: true },
)

const { selectedIds, selectedItems, clearSelection, removeFromSelection } = useContentSelection(
	ctx.items,
	getItemId,
)

const bulk = useBulkOperation()
const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, bulkWaiting } = bulk

// Sync bulk operation state back to the content manager so providers can suppress refreshes
if (ctx.isBulkOperating) {
	watch(isBulkOperating, (val) => {
		ctx.isBulkOperating!.value = val
	})
}

const { isChanging, markChanging, unmarkChanging } = useChangingItems()

const bulkStatusMessage = ref<string | null>(null)

const refreshing = ref(false)
async function handleRefresh() {
	if (refreshing.value) return
	retainedRowIds.value.clear()
	refreshing.value = true
	try {
		await ctx.refresh()
	} finally {
		refreshing.value = false
	}
}

const filteredItems = computed(() => {
	const sorted = sortedItems.value
	const searched = search(sorted)
	const matching = applyMetadataFilters(searched)
	if (retainedRowIds.value.size === 0) return matching
	const matchingIds = new Set(matching.map(getItemId))
	return searched.filter((item) => {
		const id = getItemId(item)
		return matchingIds.has(id) || retainedRowIds.value.has(id)
	})
})
const tableItems = computed<ContentCardTableItem[]>(() => {
	const items = filteredItems.value.map((item) => {
		const base = ctx.mapToTableItem(item)
		const id = getItemId(item)
		const mutationPending =
			ctx.disableWhileMutating !== false && (isChanging(id) || isBulkOperating.value)
		const locked = base.locked ?? item.locked ?? false
		const clientWarning = base.enabledFor
			? null
			: getClientWarningType(item, ctx.showEnvironmentWarnings)
		return {
			...base,
			id,
			projectType: item.project_type,
			locked,
			disabled: mutationPending || ctx.isBusy.value || item.installing === true,
			disabledTooltip: ctx.isBusy.value
				? (ctx.busyMessage?.value ?? null)
				: mutationPending
					? formatMessage(messages.pleaseWait)
					: item.installing
						? formatMessage(commonMessages.installingLabel)
						: null,
			toggleDisabled: ctx.isBusy.value || base.toggleDisabled,
			toggleDisabledTooltip: ctx.isBusy.value
				? (ctx.busyMessage?.value ?? null)
				: base.toggleDisabledTooltip,
			installing: item.installing === true,
			installProgress: item.installProgress,
			hasUpdate: base.hasUpdate ?? item.has_update,
			clientWarning,
			hideDelete: base.hideDelete,
			hideSwitchVersion: base.hideSwitchVersion ?? !base.versionLink,
			overflowOptions: ctx.getOverflowOptions?.(item),
		}
	})

	const updatable = items.filter((i) => i.hasUpdate)
	if (updatable.length > 0) {
		debug('tableItems: items with hasUpdate=true', {
			count: updatable.length,
			ids: updatable.map((i) => i.id),
			isPackLocked: ctx.isPackLocked.value,
		})
	}

	return items
})

const hasOutdatedProjects = computed(() => {
	const outdated = ctx.items.value.filter((p) => p.has_update && !p.locked)
	if (outdated.length > 0) {
		debug('hasOutdatedProjects: raw items with has_update=true', {
			count: outdated.length,
			items: outdated.map((p) => ({
				id: p.id,
				fileName: p.file_name,
				title: p.project?.title,
				has_update: p.has_update,
				update_version_id: p.update_version_id,
			})),
		})
	}
	return outdated.length > 0
})

const {
	confirmDisableModal,
	pendingDisableItems,
	pendingDisableWarning,
	toggleableSelectedItems,
	promptDisableItems,
	confirmDisable,
	handleToggleEnabledById,
	bulkEnable,
	bulkDisable,
} = useContentToggleFlow({
	ctx,
	getItemId,
	selectedItems,
	clearSelection,
	bulk,
	markChanging,
	unmarkChanging,
	retainedRowIds,
})

const {
	confirmDeletionModal,
	contentDependencyWarningModal,
	pendingDeletionItems,
	pendingDeletionWarning,
	pendingDependencyWarningItems,
	pendingDependencyWarningDependents,
	deletableSelectedItems,
	handleDeleteById,
	showBulkDeleteModal,
	confirmDelete,
	confirmDependencyWarningDelete,
} = useContentDeleteFlow({
	ctx,
	getItemId,
	selectedItems,
	clearSelection,
	removeFromSelection,
	bulk,
	markChanging,
	unmarkChanging,
	promptDisableItems,
})

function handleUpdateById(id: string) {
	const item = ctx.items.value.find((item) => getItemId(item) === id)
	if (!item || item.locked) return
	ctx.updateItem?.(id)
}

function handleSwitchVersionById(id: string) {
	const item = ctx.items.value.find((i) => getItemId(i) === id)
	if (item && !item.locked) {
		ctx.switchVersion?.(item)
	}
}

async function handleSetEnabledForById(id: string, side: 'server' | 'player', enabled: boolean) {
	if (ctx.isBusy.value || !ctx.setEnabledFor) return
	const item = ctx.items.value.find((candidate) => getItemId(candidate) === id)
	if (!item) return

	retainedRowIds.value.add(id)
	await ctx.setEnabledFor(item, side, enabled)
}

const {
	updateAllModal,
	updateAllItems,
	loadingUpdateAll,
	loadingUpdateAllChangelog,
	bulkItemCount,
	bulkUpdateItems,
	hasBulkUpdateSupport,
	loadUpdateAllChangelog,
	preloadUpdateAllChangelog,
	updateAllSelected,
	promptUpdateAll,
	cancelUpdateAll,
	promptUpdateSelected,
} = useContentUpdateAllFlow({
	ctx,
	getItemId,
	selectedItems,
	clearSelection,
	bulk,
	updateItemById: handleUpdateById,
})

const confirmUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
</script>

<template>
	<div class="flex flex-col gap-4" :class="{ 'pb-6': props.bottomPadding }">
		<template v-if="!ctx.loading.value">
			<div
				v-if="ctx.error.value"
				class="flex w-full flex-col items-center justify-center gap-4 p-4"
			>
				<div class="universal-card flex flex-col items-center gap-4 p-6">
					<h2 class="m-0 text-xl font-bold">{{ formatMessage(messages.failedToLoad) }}</h2>
					<p class="text-secondary">{{ ctx.error.value.message }}</p>
					<Button type="colored" color="brand" @click="handleRefresh">{{
						formatMessage(commonMessages.retryButton)
					}}</Button>
				</div>
			</div>

			<template v-else>
				<ManagedContentCard
					v-if="ctx.managed.value"
					:data="ctx.managed.value.card"
					:disabled="!!ctx.managed.value.disabledReason"
					:disabled-text="ctx.managed.value.disabledReason"
					:show-view-content="!!ctx.managed.value.view"
					:show-settings="!!ctx.managed.value.openSettings"
					:show-primary-action="!!ctx.managed.value.primaryAction"
					@view-content="ctx.managed.value?.view?.()"
					@settings="ctx.managed.value?.openSettings?.()"
					@primary-action="ctx.managed.value?.primaryAction?.($event)"
				/>

				<template v-if="ctx.items.value.length > 0">
					<div class="flex flex-col gap-4">
						<span v-if="ctx.managed.value" class="text-2xl font-semibold leading-8 text-contrast">
							{{ formatMessage(messages.additionalContent) }}
						</span>

						<ContentToolbar
							v-model:search="searchQuery"
							v-model:type="activeType"
							v-model:filters="selectedMetadataFilters"
							v-model:sort="sortMode"
							:search-placeholder="
								formatMessage(messages.searchPlaceholder, {
									count: typeItems.length,
									contentType: formatContentTypeSentence(
										formatMessage,
										activeType,
										typeItems.length,
									),
								})
							"
							:types="types"
							:add-disabled="!!ctx.busy.value || !ctx.canAdd.value"
							:add-tooltip="addTooltip"
							:show-update-all="hasBulkUpdateSupport && hasOutdatedProjects"
							:update-all-disabled="isBulkOperating || loadingUpdateAll"
							:refreshing="refreshing"
							:filter-categories="metadataFilterCategories"
							:show-date-sort="hasDateAdded"
							@add="ctx.browse(activeType)"
							@upload="ctx.upload()"
							@update-all="promptUpdateAll"
							@refresh="handleRefresh"
						/>

						<ContentTable
							v-model:selected-ids="selectedIds"
							:sections="[{ id: 'content', items: tableItems }]"
							:columns="{
								selection: true,
								enabledFor: !!ctx.setEnabledFor,
								actions: ['update', 'switchVersion', 'toggle', 'remove'],
							}"
							:highlighted-item-id="highlightedItemId"
							@toggle="handleToggleEnabledById"
							@enabled-for="handleSetEnabledForById"
							@remove="handleDeleteById"
							@update="handleUpdateById"
							@switch-version="handleSwitchVersionById"
						>
							<template #empty>
								<span>{{
									formatMessage(
										typeItems.length > 0 ? messages.noContentFound : messages.noTypeInstalled,
										{ contentType: formatContentTypeSentence(formatMessage, activeType, 2) },
									)
								}}</span>
							</template>
						</ContentTable>
					</div>
				</template>

				<ContentEmptyState
					v-else
					:heading="
						formatMessage(
							ctx.managed.value ? messages.noExtraContentInstalled : messages.noContentInstalled,
						)
					"
					:description="
						formatMessage(ctx.managed.value ? messages.emptyModpackHint : messages.emptyHint)
					"
					:add-disabled="!!ctx.busy.value || !ctx.canAdd.value"
					:add-tooltip="addTooltip"
					@add="ctx.browse(activeType)"
					@upload="ctx.upload()"
				/>
			</template>
		</template>

		<ContentSelectionBar
			v-if="!ctx.bulkUpdatesInBackground || bulkOperation !== 'update'"
			:selected-items="bulkOperation === 'update' ? bulkUpdateItems : selectedItems"
			:content-type-label="ctx.contentTypeLabel.value"
			:is-busy="ctx.isBusy.value"
			:busy-tooltip="ctx.busyMessage?.value"
			:is-bulk-operating="isBulkOperating"
			:bulk-operation="bulkOperation"
			:bulk-progress="bulkProgress"
			:bulk-total="bulkTotal"
			:bulk-waiting="bulkWaiting"
			:bulk-status-message="bulkStatusMessage"
			:bulk-item-count="bulkItemCount"
			:aria-label="formatMessage(commonMessages.selectionActionsLabel)"
			:get-item-id="getItemId"
			:toggle-items="toggleableSelectedItems"
			@clear="clearSelection"
			@enable="bulkEnable"
			@disable="bulkDisable"
		>
			<template #actions>
				<Button
					v-if="hasBulkUpdateSupport && selectedItems.some((m) => m.has_update && !m.locked)"
					v-tooltip="formatMessage(commonMessages.updateButton)"
					type="quiet"
					color="green"
					:disabled="loadingUpdateAll"
					class="hover:!bg-green focus-visible:!bg-green hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
					@click="promptUpdateSelected"
				>
					<DownloadIcon />
					<span class="bar-label">{{ formatMessage(commonMessages.updateButton) }}</span>
				</Button>

				<TeleportOverflowMenu
					v-if="ctx.shareItems"
					type="quiet"
					:label="formatMessage(commonMessages.moreOptionsButton)"
					:options="[
						{
							id: 'share-names',
							label: formatMessage(messages.shareProjectNames),
							action: () => ctx.shareItems!(selectedItems, 'names'),
						},
						{
							id: 'share-file-names',
							label: formatMessage(messages.shareFileNames),
							action: () => ctx.shareItems!(selectedItems, 'file-names'),
						},
						{
							id: 'share-urls',
							label: formatMessage(messages.shareProjectLinks),
							action: () => ctx.shareItems!(selectedItems, 'urls'),
						},
						{
							id: 'share-markdown',
							label: formatMessage(messages.shareMarkdownLinks),
							action: () => ctx.shareItems!(selectedItems, 'markdown'),
						},
					]"
					class="!w-auto !px-2.5 !rounded-xl"
				>
					<ShareIcon />
					<span class="bar-label">{{ formatMessage(messages.share) }}</span>
					<DropdownIcon />
					<template #share-names>
						<TextCursorInputIcon />
						{{ formatMessage(messages.shareProjectNames) }}
					</template>
					<template #share-file-names>
						<FileIcon />
						{{ formatMessage(messages.shareFileNames) }}
					</template>
					<template #share-urls>
						<LinkIcon />
						{{ formatMessage(messages.shareProjectLinks) }}
					</template>
					<template #share-markdown>
						<CodeIcon />
						{{ formatMessage(messages.shareMarkdownLinks) }}
					</template>
				</TeleportOverflowMenu>
			</template>

			<template #actions-end>
				<div v-if="deletableSelectedItems.length > 0" class="mx-1 h-6 w-px bg-surface-5" />

				<Button
					v-if="deletableSelectedItems.length > 0"
					v-tooltip="formatMessage(commonMessages.deleteLabel)"
					type="quiet"
					color="red"
					class="hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
					@click="showBulkDeleteModal"
				>
					<TrashIcon />
					<span class="bar-label">{{ formatMessage(commonMessages.deleteLabel) }}</span>
				</Button>
			</template>
		</ContentSelectionBar>

		<ConfirmDeletionModal
			ref="confirmDeletionModal"
			:count="pendingDeletionItems.length"
			:item-type="ctx.contentTypeLabel.value"
			:warning="pendingDeletionWarning"
			:variant="ctx.deletionContext ?? 'instance'"
			:backup-tip="pendingDeletionItems.map((i) => i.project?.title ?? i.file_name).join(', ')"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@delete="confirmDelete"
		/>
		<ConfirmDisableModal
			ref="confirmDisableModal"
			:count="pendingDisableItems.length"
			:item-type="ctx.contentTypeLabel.value"
			:warning="pendingDisableWarning"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@disable="confirmDisable"
		/>
		<ContentDependencyWarningModal
			ref="contentDependencyWarningModal"
			:items="pendingDependencyWarningItems"
			:dependents="pendingDependencyWarningDependents"
			:item-type="ctx.contentTypeLabel.value"
			:variant="ctx.deletionContext ?? 'instance'"
			:backup-tip="pendingDeletionItems.map((i) => i.project?.title ?? i.file_name).join(', ')"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@delete="confirmDependencyWarningDelete"
		/>
		<UpdateAllModal
			v-if="hasBulkUpdateSupport"
			ref="updateAllModal"
			:items="updateAllItems"
			:server="ctx.deletionContext === 'server'"
			:loading="loadingUpdateAll"
			:loading-changelog="loadingUpdateAllChangelog"
			:action-disabled="ctx.isBusy.value"
			@changelog="loadUpdateAllChangelog"
			@preload-changelog="preloadUpdateAllChangelog"
			@cancel="cancelUpdateAll"
			@update="updateAllSelected"
		/>
		<ConfirmUnlinkModal
			v-if="ctx.unlinkModpack"
			ref="confirmUnlinkModal"
			:server="ctx.deletionContext === 'server'"
			:backup-tip="ctx.managedContent.value?.card.manager.name"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@unlink="ctx.unlinkModpack!()"
		/>

		<slot name="modals" />
	</div>
</template>
