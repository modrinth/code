import type { Labrinth } from '@modrinth/api-client'
import { useQueryClient } from '@tanstack/vue-query'
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

import type { ContentManagerContext } from '../../providers/content-manager'
import {
	contentVersionQueryOptions,
	getRecommendedVersionBatches,
	getUpdateCandidates,
} from '../../queries'
import type { ContentItem } from '../../types'
import type { ContentFlowState } from './types'

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

export function useUpdateAllFlow(
	ctx: ContentManagerContext,
	state: ContentFlowState,
	updateItem: (item: ContentItem) => void,
) {
	const { clearSelection, bulk } = state
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

	const hasBulkUpdateSupport = computed(() => !!ctx.applyUpdates)

	async function loadRecommendedUpdateVersions(items: ContentItem[]) {
		const batches = await Promise.all(
			getRecommendedVersionBatches(items).map(async (batch) => {
				try {
					const versions = await queryClient.fetchQuery(
						contentVersionQueryOptions.versionBatch(batch, client),
					)
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
		if (!ctx.applyUpdates || ctx.busy.value || isBulkOperating.value || loadingUpdateAll.value)
			return
		const candidates = getUpdateCandidates(items)
		if (candidates.length === 0) return
		if (candidates.length === 1) {
			updateItem(candidates[0])
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
		await Promise.all(
			projectIds.map(async (projectId) => {
				try {
					const versions = await queryClient.fetchQuery(
						contentVersionQueryOptions.projectVersions(projectId, client),
					)
					projectVersions.set(projectId, versions)
				} catch {
					failedToLoadVersions = true
				}
			}),
		)
		if (requestId !== updateAllRequestId) return
		const recommended = await recommendedPromise
		if (requestId !== updateAllRequestId) return

		updateAllItems.value = candidates.map((item) => {
			const preferredVersionId = item.update_version_id!
			const versions = projectVersions.get(item.project!.id) ?? []
			const preferredVersion =
				queryClient.getQueryData<Labrinth.Versions.v2.Version>(
					contentVersionQueryOptions.version(preferredVersionId, client).queryKey,
				) ??
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
						gameVersion: ctx.target.value.gameVersion,
						loader: ctx.target.value.loader,
						projectType: item.project_type,
					}),
			)
			return {
				id: item.id,
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
		void queryClient.prefetchQuery(contentVersionQueryOptions.version(selection.version.id, client))
	}

	async function loadUpdateAllChangelog(selection: UpdateAllSelection) {
		if (selection.version.changelog != null) return
		const requestId = updateAllRequestId
		loadingUpdateAllChangelog.value = true
		try {
			const version = await queryClient.fetchQuery(
				contentVersionQueryOptions.version(selection.version.id, client),
			)
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
		if (!ctx.applyUpdates || ctx.busy.value || isBulkOperating.value || selections.length === 0)
			return
		++updateAllRequestId
		isBulkOperating.value = true
		bulkOperation.value = 'update'
		bulkItemCount.value = selections.length
		bulkTotal.value = selections.length
		bulkProgress.value = 0
		bulkUpdateItems.value = selections.flatMap((selection) => {
			const item = ctx.items.value.find((item) => item.id === selection.id)
			return item ? [item] : []
		})
		bulkWaiting.value = true
		try {
			await ctx.applyUpdates(selections, (completed) => {
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

	function promptUpdate(items: ContentItem[]) {
		void openUpdateAll(items)
	}

	function cancelUpdateAll() {
		++updateAllRequestId
		loadingUpdateAll.value = false
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
		promptUpdate,
		cancelUpdateAll,
	}
}
