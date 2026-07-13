<template>
	<div class="flex flex-col gap-4" :class="{ 'pb-6': bottomPadding }">
		<template v-if="!ctx.loading.value">
			<div
				v-if="ctx.error.value"
				class="flex w-full flex-col items-center justify-center gap-4 p-4"
			>
				<div class="universal-card flex flex-col items-center gap-4 p-6">
					<h2 class="m-0 text-xl font-bold">{{ formatMessage(messages.failedToLoad) }}</h2>
					<p class="text-secondary">{{ ctx.error.value.message }}</p>
					<ButtonStyled color="brand">
						<button @click="handleRefresh">{{ formatMessage(commonMessages.retryButton) }}</button>
					</ButtonStyled>
				</div>
			</div>

			<template v-else>
				<ContentModpackCard
					v-if="ctx.modpack.value"
					:project="ctx.modpack.value.project"
					:project-link="ctx.modpack.value.projectLink"
					:version="ctx.modpack.value.version"
					:version-link="ctx.modpack.value.versionLink"
					:owner="ctx.modpack.value.owner"
					:categories="ctx.modpack.value.categories"
					:has-update="ctx.modpack.value.hasUpdate"
					:disabled="ctx.modpack.value.disabled"
					:disabled-text="ctx.modpack.value.disabledText"
					:show-content-hint="
						!!(ctx.showContentHint?.value && ctx.modpack.value && ctx.items.value.length === 0)
					"
					v-on="{
						...(ctx.updateModpack ? { update: () => ctx.updateModpack?.() } : {}),
						...(ctx.viewModpackContent ? { content: () => ctx.viewModpackContent?.() } : {}),
						...(ctx.unlinkModpack ? { unlink: () => confirmUnlinkModal?.show() } : {}),
						...(ctx.openSettings ? { settings: () => ctx.openSettings?.() } : {}),
					}"
					@dismiss-content-hint="ctx.dismissContentHint?.()"
				/>

				<template v-if="ctx.items.value.length > 0">
					<div class="flex flex-col gap-4">
						<span v-if="ctx.modpack.value" class="text-xl font-semibold text-contrast">
							{{ formatMessage(messages.additionalContent) }}
						</span>

						<div class="flex flex-wrap items-center gap-2">
							<StyledInput
								v-model="searchQuery"
								:icon="SearchIcon"
								type="text"
								autocomplete="off"
								:spellcheck="false"
								input-class="!h-10"
								wrapper-class="flex-1 min-w-0"
								clearable
								:placeholder="
									formatMessage(messages.searchPlaceholder, {
										count: tableItems.length,
										contentType: formatContentTypeSentence(
											formatMessage,
											ctx.contentTypeLabel.value,
											tableItems.length,
										),
									})
								"
							/>

							<div class="flex gap-2">
								<ButtonStyled color="brand">
									<button
										v-tooltip="
											ctx.busyMessage?.value ??
											(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
										"
										:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
										class="!h-10 flex items-center gap-2"
										@click="ctx.browse"
									>
										<CompassIcon class="size-5" />
										<span>{{ formatMessage(messages.browseContent) }}</span>
									</button>
								</ButtonStyled>
								<ButtonStyled type="outlined">
									<button
										v-tooltip="
											ctx.busyMessage?.value ??
											(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
										"
										:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
										class="!h-10"
										@click="ctx.uploadFiles"
									>
										<FolderOpenIcon class="size-5" />
										{{ formatMessage(messages.uploadFiles) }}
									</button>
								</ButtonStyled>
							</div>
						</div>

						<div class="@container flex flex-wrap items-center justify-between gap-2">
							<div class="flex flex-wrap items-center gap-1.5">
								<FilterIcon class="size-5 text-secondary" />
								<button
									class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
									:class="
										selectedFilters.length === 0
											? 'border-green bg-brand-highlight text-brand'
											: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
									"
									:aria-pressed="selectedFilters.length === 0"
									@click="selectedFilters = []"
								>
									{{ formatMessage(commonMessages.allProjectType) }}
								</button>
								<button
									v-for="option in filterOptions"
									:key="option.id"
									class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
									:class="
										selectedFilters.includes(option.id)
											? 'border-green bg-brand-highlight text-brand'
											: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
									"
									:aria-pressed="selectedFilters.includes(option.id)"
									@click="toggleFilter(option.id)"
								>
									{{ option.label }}
								</button>
								<div class="hidden @[900px]:block">
									<ButtonStyled type="transparent">
										<button
											:aria-label="
												formatMessage(messages.sortByLabel, { mode: sortLabels[sortMode]() })
											"
											@click="cycleSortMode"
										>
											<ArrowUpZAIcon v-if="sortMode === 'alphabetical-desc'" /><ClockArrowDownIcon
												v-else-if="sortMode === 'date-added-newest'"
											/><ClockArrowUpIcon
												v-else-if="sortMode === 'date-added-oldest'"
											/><ArrowDownAZIcon v-else />
											{{ sortLabels[sortMode]() }}
										</button>
									</ButtonStyled>
								</div>
							</div>

							<div class="flex items-center gap-2">
								<div class="@[900px]:hidden">
									<ButtonStyled type="transparent">
										<button
											:aria-label="
												formatMessage(messages.sortByLabel, { mode: sortLabels[sortMode]() })
											"
											@click="cycleSortMode"
										>
											<ArrowUpZAIcon v-if="sortMode === 'alphabetical-desc'" /><ClockArrowDownIcon
												v-else-if="sortMode === 'date-added-newest'"
											/><ClockArrowUpIcon
												v-else-if="sortMode === 'date-added-oldest'"
											/><ArrowDownAZIcon v-else />
											{{ sortLabels[sortMode]() }}
										</button>
									</ButtonStyled>
								</div>

								<ButtonStyled
									v-if="hasBulkUpdateSupport && hasOutdatedProjects"
									color="green"
									type="transparent"
									color-fill="text"
									hover-color-fill="background"
								>
									<button
										v-tooltip="formatMessage(messages.updateAll)"
										:disabled="isBulkOperating"
										@click="promptUpdateAll"
									>
										<DownloadIcon />
										{{ formatMessage(messages.updateAll) }}
									</button>
								</ButtonStyled>

								<ButtonStyled type="transparent">
									<button :disabled="refreshing" @click="handleRefresh">
										<RefreshCwIcon :class="refreshing ? 'animate-spin' : ''" />
										{{ formatMessage(commonMessages.refreshButton) }}
									</button>
								</ButtonStyled>
							</div>
						</div>

						<!-- Folder sections -->
						<template v-if="hasAnyFolders">
							<div
								v-for="folder in visibleFolders"
								:key="folder.id"
								class="border border-solid border-surface-4 rounded-[20px] overflow-clip"
							>
								<div
									class="flex items-center gap-3 px-3 py-2.5 bg-surface-3 cursor-pointer select-none"
									@click="toggleFolder(folder.id)"
								>
									<DropdownIcon
										class="size-5 text-secondary transition-transform duration-200 shrink-0"
										:class="folder.expanded ? 'rotate-0' : '-rotate-90'"
									/>
									<FolderIcon class="size-5 text-secondary shrink-0" />
									<span class="font-semibold text-contrast truncate">{{ folder.name }}</span>
									<span class="text-secondary text-sm shrink-0">
										{{ formatMessage(messages.folderCount, { count: folderItems(folder).length }) }}
									</span>
									<div class="ml-auto flex items-center gap-1" @click.stop>
										<OverflowMenu :options="getFolderMenuOptions(folder)">
											<ButtonStyled type="transparent" circular>
												<button class="!p-1.5">
													<EllipsisVerticalIcon class="size-4" />
												</button>
											</ButtonStyled>
										</OverflowMenu>
									</div>
								</div>
								<template v-if="folder.expanded">
									<ContentCardTable
										:selected-ids="getFolderSelection(folder).value"
										:items="folderTableItems(folder)"
										:show-selection="true"
										:virtualized="false"
										:flat="true"
										@update:selected-ids="
											(ids: string[]) => (getFolderSelection(folder).value = ids)
										"
										@update:enabled="handleToggleEnabledById"
										@delete="handleDeleteById"
										@update="handleUpdateById"
										@switch-version="handleSwitchVersionById"
									/>
								</template>
							</div>
						</template>

						<!-- Unassigned items -->
						<ContentCardTable
							v-if="hasAnyFolders && unassignedTableItems.length > 0"
							v-model:selected-ids="rootSelection"
							:items="unassignedTableItems"
							:show-selection="true"
							@update:enabled="handleToggleEnabledById"
							@delete="handleDeleteById"
							@update="handleUpdateById"
							@switch-version="handleSwitchVersionById"
						>
							<template #empty>
								<span>{{ formatMessage(messages.noContentFound) }}</span>
							</template>
						</ContentCardTable>

						<!-- All items table when no folders exist -->
						<ContentCardTable
							v-if="!hasAnyFolders && tableItems.length > 0"
							v-model:selected-ids="rootSelection"
							:items="tableItems"
							:show-selection="true"
							@update:enabled="handleToggleEnabledById"
							@delete="handleDeleteById"
							@update="handleUpdateById"
							@switch-version="handleSwitchVersionById"
						>
							<template #empty>
								<span>{{ formatMessage(messages.noContentFound) }}</span>
							</template>
						</ContentCardTable>
					</div>
				</template>

				<EmptyState v-else type="empty-inbox">
					<template #heading>
						{{
							formatMessage(
								ctx.modpack.value ? messages.noExtraContentInstalled : messages.noContentInstalled,
							)
						}}
					</template>
					<template #description>
						{{
							ctx.modpack.value
								? formatMessage(messages.emptyModpackHint)
								: formatMessage(messages.emptyHint, {
										contentType: formatContentTypeSentence(
											formatMessage,
											ctx.contentTypeLabel.value,
											2,
											'content',
										),
									})
						}}
					</template>
					<template #actions>
						<ButtonStyled type="outlined">
							<button
								v-tooltip="
									ctx.busyMessage?.value ??
									(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
								"
								:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
								class="!h-10"
								@click="ctx.uploadFiles"
							>
								<FolderOpenIcon class="size-5" />
								{{ formatMessage(messages.uploadFiles) }}
							</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button
								v-tooltip="
									ctx.busyMessage?.value ??
									(ctx.disableAddContent?.value ? ctx.disableAddContentTooltip : undefined)
								"
								:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
								class="!h-10 flex items-center gap-2"
								@click="ctx.browse"
							>
								<CompassIcon class="size-5" />
								<span>{{ formatMessage(messages.browseContent) }}</span>
							</button>
						</ButtonStyled>
					</template>
				</EmptyState>
			</template>
		</template>

		<ContentSelectionBar
			:selected-items="selectedItems"
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
			@clear="clearSelection"
			@enable="bulkEnable"
			@disable="bulkDisable"
		>
			<template #actions>
				<ButtonStyled
					v-if="hasBulkUpdateSupport && selectedItems.some((m) => m.has_update)"
					type="transparent"
					color="green"
					color-fill="text"
					hover-color-fill="background"
				>
					<button
						v-tooltip="formatMessage(commonMessages.updateButton)"
						@click="promptUpdateSelected"
					>
						<DownloadIcon />
						<span class="bar-label">{{ formatMessage(commonMessages.updateButton) }}</span>
					</button>
				</ButtonStyled>

				<ButtonStyled v-if="ctx.shareItems" type="transparent">
					<OverflowMenu
						:options="[
							{
								id: 'share-names',
								action: () => ctx.shareItems!(selectedItems, 'names'),
							},
							{
								id: 'share-file-names',
								action: () => ctx.shareItems!(selectedItems, 'file-names'),
							},
							{
								id: 'share-urls',
								action: () => ctx.shareItems!(selectedItems, 'urls'),
							},
							{
								id: 'share-markdown',
								action: () => ctx.shareItems!(selectedItems, 'markdown'),
							},
						]"
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
					</OverflowMenu>
				</ButtonStyled>

				<ButtonStyled
					v-if="selectedItems.length > 0"
					type="transparent"
				>
					<OverflowMenu
						:options="[
							...targetFolders.map((f) => ({
								id: formatMessage(messages.moveToFolder, { name: f.name }),
								icon: FolderIcon,
								action: () => bulkMoveToFolder(f.id),
							})),
							...(selectedItemsInFolder && targetFolders.length > 0
								? [{ divider: true }]
								: []),
							...(selectedItemsInFolder
								? [
										{
											id: formatMessage(messages.removeFromFolder),
											icon: FolderUpIcon,
											action: () => bulkRemoveFromFolder(),
										},
									]
								: []),
							{ divider: true },
							{
								id: formatMessage(messages.newFolderWithMods),
								icon: PlusIcon,
								action: () => {
									props.onBulkNewFolder(selectedItems)
									clearSelection()
								},
							},
						]"
					>
						<FolderIcon />
						<span class="bar-label">{{ formatMessage(messages.moveToFolderMenu) }}</span>
						<DropdownIcon />
					</OverflowMenu>
				</ButtonStyled>
			</template>

			<template #actions-end>
				<div class="mx-1 h-6 w-px bg-surface-5" />

				<ButtonStyled
					type="transparent"
					color="red"
					color-fill="text"
					hover-color-fill="background"
				>
					<button
						v-tooltip="formatMessage(commonMessages.deleteLabel)"
						@click="showBulkDeleteModal"
					>
						<TrashIcon />
						<span class="bar-label">{{ formatMessage(commonMessages.deleteLabel) }}</span>
					</button>
				</ButtonStyled>
			</template>
		</ContentSelectionBar>

		<ConfirmDeletionModal
			ref="confirmDeletionModal"
			:count="pendingDeletionItems.length"
			:item-type="ctx.contentTypeLabel.value"
			:variant="ctx.deletionContext ?? 'instance'"
			:backup-tip="pendingDeletionItems.map((i) => i.project?.title ?? i.file_name).join(', ')"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@delete="confirmDelete"
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
		<ConfirmBulkUpdateModal
			v-if="hasBulkUpdateSupport"
			ref="confirmBulkUpdateModal"
			:count="pendingBulkUpdateItems.length"
			:server="ctx.deletionContext === 'server'"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@update="confirmBulkUpdate"
		/>
		<ConfirmUnlinkModal
			v-if="ctx.unlinkModpack"
			ref="confirmUnlinkModal"
			:server="ctx.deletionContext === 'server'"
			:backup-tip="ctx.modpack.value?.project.title"
			:action-disabled="ctx.isBusy.value"
			:action-disabled-tooltip="ctx.busyMessage?.value ?? undefined"
			@unlink="ctx.unlinkModpack!()"
		/>

		<slot name="modals" />
	</div>
</template>

<script setup lang="ts">
import {
	ArrowDownAZIcon,
	ArrowUpZAIcon,
	ClockArrowDownIcon,
	ClockArrowUpIcon,
	CodeIcon,
	CompassIcon,
	DownloadIcon,
	DropdownIcon,
	EllipsisVerticalIcon,
	FileIcon,
	FilterIcon,
	FolderIcon,
	FolderOpenIcon,
	FolderUpIcon,
	LinkIcon,
	PlusIcon,
	RefreshCwIcon,
	SearchIcon,
	ShareIcon,
	TextCursorInputIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	type BulkOperationStatus,
	ButtonStyled,
	commonMessages,
	ConfirmBulkUpdateModal,
	ConfirmDeletionModal,
	ConfirmUnlinkModal,
	ContentCardTable,
	type ContentCardTableItem,
	ContentDependencyWarningModal,
	type ContentItem,
	ContentModpackCard,
	defineMessages,
	EmptyState,
	formatContentTypeSentence,
	OverflowMenu,
	type OverflowMenuOption,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { injectContentManager } from '@modrinth/ui'
import ContentSelectionBar from '@modrinth/ui/src/layouts/shared/content-tab/components/ContentSelectionBar.vue'
import {
	useBulkOperation,
	useChangingItems,
	useContentFilters,
	useContentSearch,
	useContentSelection,
} from '@modrinth/ui/src/layouts/shared/content-tab/composables'
import { computed, nextTick, ref, watch, type WritableComputedRef } from 'vue'

import type { ModFolder } from '@/helpers/mod-folders'

const { formatMessage } = useVIntl()
const ctx = injectContentManager()

const props = withDefaults(
	defineProps<{
		bottomPadding?: boolean
		folders: ModFolder[]
		toggleFolder: (id: string) => void
		renameFolder: (id: string) => void
		deleteFolder: (id: string) => void
		getModId: (item: ContentItem) => string
		moveModToFolder: (item: ContentItem, folderId: string) => void
		moveModToRoot: (item: ContentItem) => void
		onBulkNewFolder: (items: ContentItem[]) => void
	}>(),
	{
		bottomPadding: true,
	},
)

const messages = defineMessages({
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
	browseContent: {
		id: 'content.page-layout.browse-content',
		defaultMessage: 'Browse content',
	},
	uploadFiles: {
		id: 'content.page-layout.upload-files',
		defaultMessage: 'Upload files',
	},
	sortAlphabetical: {
		id: 'content.page-layout.sort.alphabetical',
		defaultMessage: 'Alphabetical',
	},
	sortDateAddedNewest: {
		id: 'content.page-layout.sort.date-added-newest',
		defaultMessage: 'Newest first',
	},
	sortDateAddedOldest: {
		id: 'content.page-layout.sort.date-added-oldest',
		defaultMessage: 'Oldest first',
	},
	updateAll: {
		id: 'content.page-layout.update-all',
		defaultMessage: 'Update all',
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
		id: 'content.page-layout.empty.hint',
		defaultMessage: 'Browse or upload {contentType} to get started',
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
	sortByLabel: {
		id: 'content.page-layout.sort.label',
		defaultMessage: 'Sort by {mode}',
	},
	renameFolder: {
		id: 'app.instance.mods.rename-folder',
		defaultMessage: 'Rename',
	},
	deleteFolder: {
		id: 'app.instance.mods.delete-folder',
		defaultMessage: 'Delete folder',
	},
	folderCount: {
		id: 'app.instance.mods.folder-count',
		defaultMessage: '{count, number} {count, plural, one {project} other {projects}}',
	},
	moveToFolderMenu: {
		id: 'app.instance.mods.move-to-folder-menu',
		defaultMessage: 'Move to...',
	},
	moveToFolder: {
		id: 'app.instance.mods.move-to-folder',
		defaultMessage: 'Move to {name}',
	},
	removeFromFolder: {
		id: 'app.instance.mods.remove-from-folder',
		defaultMessage: 'Remove from folder',
	},
	newFolderWithMods: {
		id: 'app.instance.mods.new-folder-with-mods',
		defaultMessage: 'New folder with these mods...',
	},
})

function getItemId(item: ContentItem) {
	return props.getModId(item)
}

type SortMode = 'alphabetical-asc' | 'alphabetical-desc' | 'date-added-newest' | 'date-added-oldest'
const sortMode = ref<SortMode>('alphabetical-asc')

const sortLabels: Record<SortMode, () => string> = {
	'alphabetical-asc': () => formatMessage(messages.sortAlphabetical),
	'alphabetical-desc': () => formatMessage(messages.sortAlphabetical),
	'date-added-newest': () => formatMessage(messages.sortDateAddedNewest),
	'date-added-oldest': () => formatMessage(messages.sortDateAddedOldest),
}

function cycleSortMode() {
	const modes: SortMode[] = [
		'alphabetical-asc',
		'alphabetical-desc',
		'date-added-newest',
		'date-added-oldest',
	]
	const idx = modes.indexOf(sortMode.value)
	sortMode.value = modes[(idx + 1) % modes.length]
}

const sortedItems = computed(() => {
	const items = [...ctx.items.value]
	switch (sortMode.value) {
		case 'alphabetical-desc':
			return items.sort((a, b) => {
				const nameA = a.project?.title ?? a.file_name
				const nameB = b.project?.title ?? b.file_name
				return (
					nameB.toLowerCase().localeCompare(nameA.toLowerCase()) ||
					a.file_name.localeCompare(b.file_name)
				)
			})
		case 'date-added-newest':
			return items.sort((a, b) => {
				const dateA = a.date_added ?? ''
				const dateB = b.date_added ?? ''
				return dateB.localeCompare(dateA) || a.file_name.localeCompare(b.file_name)
			})
		case 'date-added-oldest':
			return items.sort((a, b) => {
				const dateA = a.date_added ?? ''
				const dateB = b.date_added ?? ''
				return dateA.localeCompare(dateB) || a.file_name.localeCompare(b.file_name)
			})
		default:
			return items.sort((a, b) => {
				const nameA = a.project?.title ?? a.file_name
				const nameB = b.project?.title ?? b.file_name
				return (
					nameA.toLowerCase().localeCompare(nameB.toLowerCase()) ||
					a.file_name.localeCompare(b.file_name)
				)
			})
	}
})

const { searchQuery, search } = useContentSearch(sortedItems, [
	'project.title',
	'owner.name',
	'file_name',
])

const { selectedFilters, filterOptions, toggleFilter, applyFilters } = useContentFilters(
	ctx.items,
	{
		showTypeFilters: true,
		showUpdateFilter: ctx.hasUpdateSupport,
		showWarningsFilter: true,
		isPackLocked: ctx.isPackLocked,
		persistKey: ctx.filterPersistKey,
	},
)

const { selectedIds, selectedItems, clearSelection, removeFromSelection } = useContentSelection(
	ctx.items,
	getItemId,
)

function makeTableSelection(tableItemIds: () => string[]): WritableComputedRef<string[]> {
	return computed({
		get() {
			const ids = new Set(tableItemIds())
			return selectedIds.value.filter((id) => ids.has(id))
		},
		set(newIds: string[]) {
			const otherIds = selectedIds.value.filter((id) => !tableItemIds().includes(id))
			selectedIds.value = [...otherIds, ...newIds]
		},
	})
}

const unassignedTableItemIds = computed(() => unassignedTableItems.value.map((item) => item.id))

const rootSelection = makeTableSelection(() => unassignedTableItemIds.value)

const folderSelectionCache = new Map<string, WritableComputedRef<string[]>>()
function getFolderSelection(folder: ModFolder): WritableComputedRef<string[]> {
	let cached = folderSelectionCache.get(folder.id)
	if (!cached) {
		cached = makeTableSelection(() => {
			const currentFolder = props.folders.find((f) => f.id === folder.id)
			return currentFolder ? folderTableItems(currentFolder).map((item) => item.id) : []
		})
		folderSelectionCache.set(folder.id, cached)
	}
	return cached
}

watch(
	() => props.folders.map((f) => f.id),
	(newIds) => {
		const idSet = new Set(newIds)
		for (const key of folderSelectionCache.keys()) {
			if (!idSet.has(key)) folderSelectionCache.delete(key)
		}
	},
)

const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, runBulk } = useBulkOperation()

if (ctx.isBulkOperating) {
	watch(isBulkOperating, (val) => {
		ctx.isBulkOperating!.value = val
	})
}

const { isChanging, markChanging, unmarkChanging } = useChangingItems()

const bulkWaiting = ref(false)
const bulkStatusMessage = ref<string | null>(null)
const bulkItemCount = ref(0)

const refreshing = ref(false)
async function handleRefresh() {
	if (refreshing.value) return
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
	return applyFilters(searched)
})

const tableItems = computed<ContentCardTableItem[]>(() => {
	return filteredItems.value.map((item) => mapItemToTable(item))
})

function mapItemToTable(item: ContentItem): ContentCardTableItem {
	const base = ctx.mapToTableItem(item)
	const id = getItemId(item)
	return {
		...base,
		id,
		disabled:
			isChanging(id) || ctx.isBusy.value || isBulkOperating.value || item.installing === true,
		disabledTooltip: ctx.isBusy.value ? (ctx.busyMessage?.value ?? null) : null,
		toggleDisabled: ctx.isBusy.value,
		toggleDisabledTooltip: ctx.isBusy.value ? (ctx.busyMessage?.value ?? null) : null,
		installing: item.installing === true,
		hasUpdate: item.has_update,
		isClientOnly: false,
		clientWarning: null,
		hideSwitchVersion: !base.versionLink,
		overflowOptions: ctx.getOverflowOptions?.(item),
	}
}

const hasAnyFolders = computed(() => props.folders.length > 0)

const selectedItemsInFolder = computed(() =>
	selectedItems.value.some((item) => {
		const id = getItemId(item)
		return props.folders.some((f) => f.modIds.includes(id))
	}),
)

function bulkMoveToFolder(folderId: string) {
	for (const item of selectedItems.value) {
		props.moveModToFolder(item, folderId)
	}
	clearSelection()
}

function bulkRemoveFromFolder() {
	for (const item of selectedItems.value) {
		props.moveModToRoot(item)
	}
	clearSelection()
}

const targetFolders = computed(() =>
	props.folders.filter(
		(f) => !selectedItems.value.every((item) => f.modIds.includes(getItemId(item))),
	),
)

const visibleFolders = computed(() => {
	const hasSearch = searchQuery.value.trim().length > 0
	const hasFilters = selectedFilters.value.length > 0
	if (!hasSearch && !hasFilters) return props.folders
	return props.folders.filter((f) => folderItems(f).length > 0)
})

function folderItems(folder: ModFolder): ContentItem[] {
	const idSet = new Set(folder.modIds)
	return filteredItems.value.filter((item) => idSet.has(getItemId(item)))
}

function folderTableItems(folder: ModFolder): ContentCardTableItem[] {
	return folderItems(folder).map((item) => mapItemToTable(item))
}

const unassignedItems = computed(() => {
	const allAssignedIds = new Set(props.folders.flatMap((f) => f.modIds))
	return filteredItems.value.filter((item) => !allAssignedIds.has(getItemId(item)))
})

const unassignedTableItems = computed<ContentCardTableItem[]>(() => {
	return unassignedItems.value.map((item) => mapItemToTable(item))
})

const hasOutdatedProjects = computed(() => ctx.items.value.some((p) => p.has_update))

function getFolderMenuOptions(folder: ModFolder): OverflowMenuOption[] {
	return [
		{
			id: formatMessage(messages.renameFolder),
			action: () => props.renameFolder(folder.id),
		},
		{
			id: formatMessage(messages.deleteFolder),
			color: 'red',
			action: () => props.deleteFolder(folder.id),
		},
	]
}

// Deletion
const pendingDeletionItems = ref<ContentItem[]>([])
const confirmDeletionModal = ref<InstanceType<typeof ConfirmDeletionModal>>()
const contentDependencyWarningModal = ref<InstanceType<typeof ContentDependencyWarningModal>>()
const pendingDependencyWarningItems = ref<ContentCardTableItem[]>([])
const pendingDependencyWarningDependents = ref<
	Array<{
		item: ContentCardTableItem
		dependencies: ContentCardTableItem[]
	}>
>([])
const pendingDependencyWarningDisableTargets = ref<ContentItem[]>([])

function mapToDisplayItem(item: ContentItem) {
	return {
		...ctx.mapToTableItem(item),
		id: getItemId(item),
	}
}

async function promptDeleteItems(items: ContentItem[], event?: MouseEvent) {
	if (items.length === 0) return
	pendingDeletionItems.value = items
	pendingDependencyWarningItems.value = []
	pendingDependencyWarningDependents.value = []
	pendingDependencyWarningDisableTargets.value = []
	const deletingIds = new Set(items.map(getItemId))

	const warning = ctx.getDeleteDependencyWarning
		? await Promise.resolve()
				.then(() => ctx.getDeleteDependencyWarning!(items))
				.catch(() => null)
		: null
	if (warning) {
		const remainingDependents = warning.dependents.filter(
			(dependent) => !deletingIds.has(getItemId(dependent.item)),
		)

		if (remainingDependents.length === 0) {
			showDeletionConfirmation(event)
			return
		}

		const relevantDependencyIds = new Set(
			remainingDependents.flatMap((dependent) => dependent.dependencies.map(getItemId)),
		)
		const warningItems = items.filter((item) => relevantDependencyIds.has(getItemId(item)))
		if (warningItems.length === 0) {
			showDeletionConfirmation(event)
			return
		}

		pendingDependencyWarningItems.value = warningItems.map(mapToDisplayItem)
		pendingDependencyWarningDependents.value = remainingDependents.map((dependent) => ({
			item: mapToDisplayItem(dependent.item),
			dependencies: dependent.dependencies
				.filter((dependency) => relevantDependencyIds.has(getItemId(dependency)))
				.map(mapToDisplayItem),
		}))
		pendingDependencyWarningDisableTargets.value = remainingDependents.map(
			(dependent) => dependent.item,
		)
		contentDependencyWarningModal.value?.show()
		return
	}

	showDeletionConfirmation(event)
}

async function showDeletionConfirmation(event?: MouseEvent) {
	if ((event?.shiftKey || skipNonEssentialWarnings.value) && !ctx.isBusy.value) {
		confirmDelete()
	} else {
		await nextTick()
		confirmDeletionModal.value?.show()
	}
}

async function handleDeleteById(id: string, event?: MouseEvent) {
	const item = ctx.items.value.find((i) => getItemId(i) === id)
	if (item) {
		await promptDeleteItems([item], event)
	}
}

async function showBulkDeleteModal(event?: MouseEvent) {
	await promptDeleteItems([...selectedItems.value], event)
}

async function confirmDependencyWarningDelete(disableDependentsAfterDeleting: boolean) {
	if (disableDependentsAfterDeleting) {
		pendingDependencyWarningDisableTargets.value =
			pendingDependencyWarningDisableTargets.value.filter((item) => item.enabled)
	} else {
		pendingDependencyWarningDisableTargets.value = []
	}

	pendingDependencyWarningItems.value = []
	pendingDependencyWarningDependents.value = []
	await confirmDelete()
}

async function disablePendingDependencyWarningDependents() {
	const items = pendingDependencyWarningDisableTargets.value.filter((item) => item.enabled)
	pendingDependencyWarningDisableTargets.value = []
	if (items.length === 0) return

	if (ctx.bulkDisableItems) {
		await ctx.bulkDisableItems(items)
		return
	}

	for (const item of items) {
		const id = getItemId(item)
		markChanging(id)
		try {
			await ctx.toggleEnabled(item)
		} finally {
			unmarkChanging(id)
		}
	}
}

async function confirmDelete() {
	if (ctx.isBusy.value) return
	const itemsToDelete = [...pendingDeletionItems.value]
	pendingDeletionItems.value = []
	if (itemsToDelete.length === 0) return

	if (ctx.bulkDeleteItems && itemsToDelete.length > 1) {
		isBulkOperating.value = true
		bulkOperation.value = 'delete'
		bulkProgress.value = 0
		bulkTotal.value = itemsToDelete.length
		bulkWaiting.value = true
		try {
			await ctx.bulkDeleteItems(itemsToDelete)
			await disablePendingDependencyWarningDependents()
		} finally {
			clearSelection()
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkProgress.value = 0
			bulkTotal.value = 0
			bulkWaiting.value = false
		}
		return
	}

	if (itemsToDelete.length === 1) {
		const item = itemsToDelete[0]
		const id = getItemId(item)
		markChanging(id)
		try {
			await ctx.deleteItem(item)
			removeFromSelection(id)
			await disablePendingDependencyWarningDependents()
		} finally {
			unmarkChanging(id)
		}
		return
	}

	await runBulk(
		'delete',
		itemsToDelete,
		async (item) => {
			await ctx.deleteItem(item)
			removeFromSelection(getItemId(item))
		},
		{ onComplete: clearSelection },
	)
	await disablePendingDependencyWarningDependents()
}

async function handleToggleEnabledById(id: string, _value: boolean) {
	if (ctx.isBusy.value) return
	const item = ctx.items.value.find((i) => getItemId(i) === id)
	if (!item) return
	markChanging(id)
	try {
		await ctx.toggleEnabled(item)
	} finally {
		unmarkChanging(id)
	}
}

async function bulkEnable() {
	if (ctx.isBusy.value) return
	const items = selectedItems.value.filter((item) => !item.enabled)
	if (items.length === 0) return
	if (ctx.bulkEnableItems) {
		isBulkOperating.value = true
		bulkOperation.value = 'enable'
		bulkProgress.value = 0
		bulkTotal.value = items.length
		bulkWaiting.value = true
		try {
			await ctx.bulkEnableItems(items)
		} finally {
			clearSelection()
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkProgress.value = 0
			bulkTotal.value = 0
			bulkWaiting.value = false
		}
		return
	}
	await runBulk('enable', items, (item) => ctx.toggleEnabled(item), { onComplete: clearSelection })
}

async function bulkDisable() {
	if (ctx.isBusy.value) return
	const items = selectedItems.value.filter((item) => item.enabled)
	if (items.length === 0) return
	if (ctx.bulkDisableItems) {
		isBulkOperating.value = true
		bulkOperation.value = 'disable'
		bulkProgress.value = 0
		bulkTotal.value = items.length
		bulkWaiting.value = true
		try {
			await ctx.bulkDisableItems(items)
		} finally {
			clearSelection()
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkProgress.value = 0
			bulkTotal.value = 0
			bulkWaiting.value = false
		}
		return
	}
	await runBulk('disable', items, (item) => ctx.toggleEnabled(item), { onComplete: clearSelection })
}

function handleUpdateById(id: string) {
	ctx.updateItem?.(id)
}

function handleSwitchVersionById(id: string) {
	const item = ctx.items.value.find((i) => getItemId(i) === id)
	if (item) {
		ctx.switchVersion?.(item)
	}
}

const skipNonEssentialWarnings = computed(() => ctx.skipNonEssentialWarnings?.value ?? false)

// Bulk updating
const confirmBulkUpdateModal = ref<InstanceType<typeof ConfirmBulkUpdateModal>>()
const pendingBulkUpdateItems = ref<ContentItem[]>([])
const pendingBulkUpdateAll = ref(false)

const hasBulkUpdateSupport = computed(
	() => !!(ctx.bulkUpdateAll || ctx.bulkUpdateItem || ctx.bulkUpdateItems),
)

function promptUpdateAll(event?: MouseEvent) {
	if (!hasBulkUpdateSupport.value) return
	const items = ctx.items.value.filter((item) => item.has_update)
	if (items.length === 0) return
	pendingBulkUpdateItems.value = items
	pendingBulkUpdateAll.value = true
	if ((event?.shiftKey || skipNonEssentialWarnings.value) && !ctx.isBusy.value) {
		confirmBulkUpdate()
	} else {
		confirmBulkUpdateModal.value?.show()
	}
}

function promptUpdateSelected(event?: MouseEvent) {
	if (!hasBulkUpdateSupport.value) return
	const items = selectedItems.value.filter((item) => item.has_update)
	if (items.length === 0) return
	pendingBulkUpdateItems.value = items
	pendingBulkUpdateAll.value = false
	if ((event?.shiftKey || skipNonEssentialWarnings.value) && !ctx.isBusy.value) {
		confirmBulkUpdate()
	} else {
		confirmBulkUpdateModal.value?.show()
	}
}

async function confirmBulkUpdate() {
	if (ctx.isBusy.value) return
	const items = pendingBulkUpdateItems.value
	if (items.length === 0 || !hasBulkUpdateSupport.value) return

	const setBulkStatus = (status: BulkOperationStatus) => {
		bulkStatusMessage.value = status.message ?? null
		bulkProgress.value = status.progress ?? bulkProgress.value
		bulkTotal.value = status.total ?? bulkTotal.value
		bulkWaiting.value = status.waiting ?? false
	}

	try {
		if (pendingBulkUpdateAll.value && ctx.bulkUpdateAll) {
			isBulkOperating.value = true
			bulkOperation.value = 'update'
			bulkProgress.value = 0
			bulkTotal.value = items.length
			bulkItemCount.value = items.length
			bulkStatusMessage.value = null
			bulkWaiting.value = true
			try {
				await ctx.bulkUpdateAll(setBulkStatus)
			} finally {
				clearSelection()
				isBulkOperating.value = false
				bulkOperation.value = null
				bulkProgress.value = 0
				bulkTotal.value = 0
				bulkItemCount.value = 0
				bulkStatusMessage.value = null
				bulkWaiting.value = false
			}
		} else if (ctx.bulkUpdateItems) {
			isBulkOperating.value = true
			bulkOperation.value = 'update'
			bulkProgress.value = 0
			bulkTotal.value = items.length
			bulkItemCount.value = items.length
			bulkStatusMessage.value = null
			bulkWaiting.value = true
			try {
				await ctx.bulkUpdateItems(items)
			} finally {
				clearSelection()
				isBulkOperating.value = false
				bulkOperation.value = null
				bulkProgress.value = 0
				bulkTotal.value = 0
				bulkItemCount.value = 0
				bulkStatusMessage.value = null
				bulkWaiting.value = false
			}
		} else if (ctx.bulkUpdateItem) {
			await runBulk('update', items, ctx.bulkUpdateItem, { onComplete: clearSelection })
		}
	} finally {
		pendingBulkUpdateItems.value = []
		pendingBulkUpdateAll.value = false
	}
}

const confirmUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
</script>
