import type { Labrinth } from '@modrinth/api-client'
import { useQueryClient } from '@tanstack/vue-query'
import { chunk } from 'es-toolkit'
import type { Ref } from 'vue'
import { computed, nextTick, ref } from 'vue'

import type UpdateAllModal from '#ui/components/modal/update-all-modal/index.vue'
import type {
	UpdateAllItem,
	UpdateAllSelection,
} from '#ui/components/modal/update-all-modal/update-all-modal-types'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient } from '#ui/providers/api-client'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { commonMessages } from '#ui/utils/common-messages'
import { versionMatchesCompatibilityTarget } from '#ui/utils/version-compatibility'

import type { ContentManagerContext } from '../providers/content-manager'
import type { ContentItem } from '../types'
import type { BulkOperationState } from './bulk-operations'

const messages = defineMessages({
	failedToLoadUpdates: {
		id: 'content.page-layout.failed-to-load-updates',
		defaultMessage: 'Failed to load available updates',
	},
	failedToLoadChangelog: {
		id: 'content.page-layout.failed-to-load-changelog',
		defaultMessage: 'Failed to load changelog',
	},
})

export interface ContentUpdateAllFlowOptions {
	ctx: ContentManagerContext
	getItemId: (item: ContentItem) => string
	selectedItems: Ref<ContentItem[]>
	clearSelection: () => void
	bulk: BulkOperationState
	updateItemById: (id: string) => void
}

export function useContentUpdateAllFlow({
	ctx,
	getItemId,
	selectedItems,
	clearSelection,
	bulk,
	updateItemById,
}: ContentUpdateAllFlowOptions) {
	const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, bulkWaiting } = bulk

	const { formatMessage } = useVIntl()
	const queryClient = useQueryClient()
	const client = injectModrinthClient()
	const { addNotification } = injectNotificationManager()

	const updateAllModal = ref<InstanceType<typeof UpdateAllModal>>()
	const updateAllItems = ref<UpdateAllItem[]>([])
	const loadingUpdateAll = ref(false)
	const loadingUpdateAllChangelog = ref(false)
	const bulkItemCount = ref(0)
	const bulkUpdateItems = ref<ContentItem[]>([])
	let updateAllRequestId = 0

	const hasBulkUpdateSupport = computed(() => !!ctx.bulkUpdateSelections)

	function getUpdateAllCandidates(items: ContentItem[]) {
		return items.filter(
			(item) =>
				item.has_update &&
				!item.locked &&
				item.project?.id &&
				item.version?.id &&
				item.update_version_id,
		)
	}

	async function loadRecommendedUpdateVersions(items: ContentItem[]) {
		const ids = [...new Set(items.map((item) => item.update_version_id!))].sort()
		const batches = await Promise.all(
			chunk(ids, 100).map(async (batch) => {
				try {
					const versions = await queryClient.fetchQuery({
						queryKey: ['labrinth', 'versions', 'v2', 'recommended', batch],
						queryFn: () =>
							client.labrinth.versions_v2.getVersions(batch, { include_changelog: false }),
						staleTime: 5 * 60_000,
					})
					for (const version of versions) {
						queryClient.setQueryData(['labrinth', 'version', 'v2', 'summary', version.id], version)
					}
					return versions
				} catch {
					return []
				}
			}),
		)
		return new Map(batches.flat().map((version) => [version.id, version]))
	}

	async function openUpdateAll(items: ContentItem[]) {
		if (
			!ctx.bulkUpdateSelections ||
			ctx.isBusy.value ||
			isBulkOperating.value ||
			loadingUpdateAll.value
		)
			return
		const candidates = getUpdateAllCandidates(items)
		if (candidates.length === 0) return
		if (candidates.length === 1 && ctx.updateItem) {
			updateItemById(getItemId(candidates[0]))
			return
		}

		const requestId = ++updateAllRequestId
		updateAllItems.value = []
		loadingUpdateAll.value = true
		loadingUpdateAllChangelog.value = false
		await nextTick()
		if (requestId !== updateAllRequestId) return
		updateAllModal.value?.show()

		const recommendedPromise = loadRecommendedUpdateVersions(candidates)
		const projectVersions = new Map<string, Labrinth.Versions.v2.Version[]>()
		let failedToLoadVersions = false
		const projectIds = [...new Set(candidates.map((item) => item.project!.id))]
		for (const batch of chunk(projectIds, 8)) {
			await Promise.all(
				batch.map(async (projectId) => {
					try {
						const versions = await queryClient.fetchQuery({
							queryKey: ['labrinth', 'versions', 'v2', projectId],
							queryFn: () =>
								client.labrinth.versions_v2.getProjectVersions(projectId, {
									include_changelog: false,
								}),
							staleTime: 5 * 60_000,
						})
						projectVersions.set(projectId, versions)
					} catch {
						failedToLoadVersions = true
					}
				}),
			)
			if (requestId !== updateAllRequestId) return
		}
		const recommended = await recommendedPromise
		if (requestId !== updateAllRequestId) return

		updateAllItems.value = candidates.map((item) => {
			const preferredVersionId = item.update_version_id!
			const versions = projectVersions.get(item.project!.id) ?? []
			const preferredVersion =
				queryClient.getQueryData<Labrinth.Versions.v2.Version>([
					'labrinth',
					'version',
					'v2',
					preferredVersionId,
				]) ??
				recommended.get(preferredVersionId) ??
				versions.find((version) => version.id === preferredVersionId)
			const sorted = [
				...versions,
				...(preferredVersion && !versions.some((version) => version.id === preferredVersion.id)
					? [preferredVersion]
					: []),
			].sort((a, b) => Date.parse(b.date_published) - Date.parse(a.date_published))
			const currentIndex = sorted.findIndex((version) => version.id === item.version!.id)
			const newer = currentIndex < 0 ? sorted : sorted.slice(0, currentIndex)
			if (preferredVersion && !newer.some((version) => version.id === preferredVersion.id)) {
				newer.unshift(preferredVersion)
			}
			const compatible = newer.filter(
				(version) =>
					version.id === preferredVersionId ||
					versionMatchesCompatibilityTarget(version, {
						gameVersion: ctx.currentGameVersion?.value ?? '',
						loader: ctx.currentLoader?.value ?? '',
						projectType: item.project_type,
					}),
			)
			return {
				id: getItemId(item),
				project: item.project!,
				currentVersion: item.version!,
				versions:
					compatible.length > 0
						? compatible
						: [
								preferredVersion ?? {
									id: preferredVersionId,
									version_number: formatMessage(commonMessages.updateAvailableLabel),
								},
							],
				initialVersionId: preferredVersionId,
			}
		})
		loadingUpdateAll.value = false
		if (failedToLoadVersions) {
			addNotification({ type: 'error', title: formatMessage(messages.failedToLoadUpdates) })
		}
	}

	function preloadUpdateAllChangelog(selection: UpdateAllSelection) {
		if (selection.version.changelog != null) return
		void queryClient.prefetchQuery({
			queryKey: ['labrinth', 'version', 'v2', selection.version.id],
			queryFn: () => client.labrinth.versions_v2.getVersion(selection.version.id),
			staleTime: 5 * 60_000,
		})
	}

	async function loadUpdateAllChangelog(selection: UpdateAllSelection) {
		if (selection.version.changelog != null) return
		const requestId = updateAllRequestId
		loadingUpdateAllChangelog.value = true
		try {
			const version = await queryClient.fetchQuery({
				queryKey: ['labrinth', 'version', 'v2', selection.version.id],
				queryFn: () => client.labrinth.versions_v2.getVersion(selection.version.id),
				staleTime: 5 * 60_000,
			})
			if (requestId !== updateAllRequestId) return
			updateAllItems.value = updateAllItems.value.map((item) => ({
				...item,
				versions: item.versions.map((candidate) =>
					candidate.id === version.id ? version : candidate,
				),
			}))
		} catch (error) {
			addNotification({
				type: 'error',
				title: formatMessage(messages.failedToLoadChangelog),
				text: error instanceof Error ? error.message : undefined,
			})
		} finally {
			if (requestId === updateAllRequestId) loadingUpdateAllChangelog.value = false
		}
	}

	async function updateAllSelected(selections: UpdateAllSelection[]) {
		if (
			!ctx.bulkUpdateSelections ||
			ctx.isBusy.value ||
			isBulkOperating.value ||
			selections.length === 0
		)
			return
		++updateAllRequestId
		isBulkOperating.value = true
		bulkOperation.value = 'update'
		bulkItemCount.value = selections.length
		bulkTotal.value = selections.length
		bulkProgress.value = 0
		bulkUpdateItems.value = selections.flatMap((selection) => {
			const item = ctx.items.value.find((item) => getItemId(item) === selection.id)
			return item ? [item] : []
		})
		bulkWaiting.value = true
		try {
			await ctx.bulkUpdateSelections(selections, (completed) => {
				bulkWaiting.value = false
				bulkProgress.value = completed
			})
			clearSelection()
		} catch {
			return
		} finally {
			isBulkOperating.value = false
			bulkOperation.value = null
			bulkItemCount.value = 0
			bulkTotal.value = 0
			bulkProgress.value = 0
			bulkUpdateItems.value = []
			bulkWaiting.value = false
		}
	}

	function promptUpdateAll() {
		void openUpdateAll(ctx.items.value)
	}

	function cancelUpdateAll() {
		++updateAllRequestId
		loadingUpdateAll.value = false
	}

	function promptUpdateSelected() {
		void openUpdateAll(selectedItems.value)
	}

	return {
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
	}
}
