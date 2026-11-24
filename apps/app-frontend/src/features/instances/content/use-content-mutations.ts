import type { ContentItem, ManagedContentModal } from '@modrinth/ui'
import { injectNotificationManager } from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import type { Ref } from 'vue'

import { instanceKeys } from '@/features/instances/queries'
import { trackEvent } from '@/helpers/analytics'
import { remove_project, set_project_locked, toggle_disable_project } from '@/helpers/instance'
import type { InstanceContentData } from '@/helpers/instance-content'
import { set_synced_pack_enabled, syncedPackKeys } from '@/helpers/synced-packs'
import type { CacheBehaviour, GameInstance } from '@/helpers/types'

import { analyticsProjectType, fileNameFromPath, matchesContentItem } from './content-items'
import type { ContentOperation, ContentOperations } from './use-content-operations'

export const contentMutationKey = ['instances', 'content'] as const

type ContentTarget = {
	mod: ContentItem
	originalFileName: string
	originalFilePath: string
	operation: ContentOperation
}

type SetEnabledVariables = {
	instanceId: string
	enabled: boolean
	targets: Array<ContentTarget & { previousEnabled: boolean }>
}

type RemoveVariables = {
	instanceId: string
	targets: ContentTarget[]
}

type FreezeVariables = {
	instanceId: string
	target: ContentTarget
	frozen: boolean
}

export interface ContentMutationsOptions {
	instance: Ref<GameInstance>
	operations: ContentOperations
	canMutateContent: (item: ContentItem) => boolean
	managedContentModal: Readonly<Ref<InstanceType<typeof ManagedContentModal> | null | undefined>>
	deleteSyncedItem: (item: ContentItem) => Promise<boolean>
	refreshContentState: (cacheBehaviour?: CacheBehaviour) => Promise<void>
	reconcileSharedInstancePublishState: () => Promise<void>
}

export function useContentMutations({
	instance,
	operations,
	canMutateContent,
	managedContentModal,
	deleteSyncedItem,
	refreshContentState,
	reconcileSharedInstancePublishState,
}: ContentMutationsOptions) {
	const { handleError } = injectNotificationManager()
	const queryClient = useQueryClient()

	function updateCachedItems(
		instanceId: string,
		{ mod, originalFileName, originalFilePath }: Omit<ContentTarget, 'operation'>,
		updates: Partial<ContentItem>,
	) {
		const matches = (item: ContentItem) =>
			matchesContentItem(item, mod, originalFileName, originalFilePath)
		managedContentModal.value?.updateItem(originalFileName, updates)
		queryClient.setQueryData<ContentItem[]>(instanceKeys.linkedContent(instanceId), (items) =>
			items?.map((item) => (matches(item) ? { ...item, ...updates } : item)),
		)
		queryClient.setQueryData<InstanceContentData>(instanceKeys.content(instanceId), (data) =>
			data
				? {
						...data,
						contentItems:
							data.contentItems?.map((item) => (matches(item) ? { ...item, ...updates } : item)) ??
							null,
					}
				: data,
		)
	}

	function cancelContentQueries(instanceId: string) {
		return Promise.all([
			queryClient.cancelQueries({ queryKey: instanceKeys.content(instanceId) }),
			queryClient.cancelQueries({ queryKey: instanceKeys.linkedContent(instanceId) }),
		])
	}

	function markContentStale(instanceId: string) {
		return Promise.all([
			queryClient.invalidateQueries({
				queryKey: instanceKeys.content(instanceId),
				refetchType: 'none',
			}),
			queryClient.invalidateQueries({
				queryKey: instanceKeys.linkedContent(instanceId),
				refetchType: 'none',
			}),
		])
	}

	function trackContentEvent(name: string, mod: ContentItem, extra: Record<string, unknown> = {}) {
		trackEvent(name, {
			loader: instance.value.loader,
			game_version: instance.value.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: analyticsProjectType(mod.project_type),
			...extra,
		})
	}

	const setEnabledMutation = useMutation({
		mutationKey: [...contentMutationKey, 'set-enabled'],
		networkMode: 'always',
		onMutate: async ({ instanceId, enabled, targets }: SetEnabledVariables) => {
			await cancelContentQueries(instanceId)
			for (const target of targets) updateCachedItems(instanceId, target, { enabled })
		},
		mutationFn: ({ instanceId, enabled, targets }: SetEnabledVariables) =>
			Promise.allSettled(
				targets.map(async ({ mod, originalFilePath }) => {
					const option = mod.project_type === 'resourcepack' ? 'resource_packs' : 'data_packs'
					if (mod.synced_pack && instance.value.synced_options[option]) {
						await set_synced_pack_enabled(mod.synced_pack.id, enabled)
						return null
					}
					return toggle_disable_project(instanceId, originalFilePath, enabled)
				}),
			),
		onSuccess: async (results, { instanceId, enabled, targets }) => {
			let syncedPackChanged = false
			let firstError: unknown = null
			for (const [index, result] of results.entries()) {
				const target = targets[index]
				if (result.status === 'rejected') {
					updateCachedItems(instanceId, target, { enabled: target.previousEnabled })
					firstError ??= result.reason
					continue
				}
				if (result.value === null) {
					syncedPackChanged = true
					continue
				}
				updateCachedItems(instanceId, target, {
					file_path: result.value,
					file_name: fileNameFromPath(result.value),
					enabled,
				})
				trackContentEvent('InstanceProjectDisable', target.mod, { disabled: !enabled })
			}
			if (firstError) handleError(firstError as Error)
			if (syncedPackChanged) {
				await refreshContentState('must_revalidate')
				await queryClient.invalidateQueries({ queryKey: syncedPackKeys.all })
			}
		},
		onError: (error, { instanceId, targets }) => {
			for (const target of targets) {
				updateCachedItems(instanceId, target, { enabled: target.previousEnabled })
			}
			handleError(error)
		},
		onSettled: (_data, _error, { instanceId }) => markContentStale(instanceId),
	})

	const removeMutation = useMutation({
		mutationKey: [...contentMutationKey, 'remove'],
		networkMode: 'always',
		onMutate: async ({ instanceId, targets }: RemoveVariables) => {
			await queryClient.cancelQueries({ queryKey: instanceKeys.content(instanceId) })
			const paths = new Set(targets.map((target) => target.originalFilePath))
			const isRemoved = (item: ContentItem) => !!item.file_path && paths.has(item.file_path)
			const removedItems =
				queryClient
					.getQueryData<InstanceContentData>(instanceKeys.content(instanceId))
					?.contentItems?.filter(isRemoved) ?? []
			updateContentList(instanceId, (items) => items.filter((item) => !isRemoved(item)))
			return { removedItems }
		},
		mutationFn: ({ instanceId, targets }: RemoveVariables) =>
			Promise.allSettled(
				targets.map(async ({ mod, originalFilePath }) => {
					if (!(await deleteSyncedItem(mod))) await remove_project(instanceId, originalFilePath)
				}),
			),
		onSuccess: async (results, { instanceId, targets }, { removedItems }) => {
			const removedPaths = new Set<string>()
			const failedPaths = new Set<string>()
			let removedSyncedPack = false
			for (const [index, result] of results.entries()) {
				const { mod, originalFilePath } = targets[index]
				if (result.status === 'rejected') {
					failedPaths.add(originalFilePath)
					handleError(result.reason as Error)
					continue
				}
				removedPaths.add(originalFilePath)
				if (mod.synced_pack) removedSyncedPack = true
				trackContentEvent('InstanceProjectRemove', mod)
			}

			updateContentList(instanceId, (items) => {
				const listedPaths = new Set(items.map((item) => item.file_path))
				return [
					...items.filter((item) => !item.file_path || !removedPaths.has(item.file_path)),
					...removedItems.filter(
						(item) => failedPaths.has(item.file_path!) && !listedPaths.has(item.file_path),
					),
				]
			})

			if (removedSyncedPack) {
				await refreshContentState('must_revalidate')
				await queryClient.invalidateQueries({ queryKey: syncedPackKeys.all })
			}
		},
		onError: (error, { instanceId }, context) => {
			if (context?.removedItems.length) {
				updateContentList(instanceId, (items) => [...items, ...context.removedItems])
			}
			handleError(error)
		},
		onSettled: (_data, _error, { instanceId }) =>
			queryClient.invalidateQueries({
				queryKey: instanceKeys.content(instanceId),
				refetchType: 'none',
			}),
	})

	const freezeMutation = useMutation({
		mutationKey: [...contentMutationKey, 'freeze'],
		networkMode: 'always',
		onMutate: async ({ instanceId, target, frozen }: FreezeVariables) => {
			await cancelContentQueries(instanceId)
			updateCachedItems(instanceId, target, { locked: frozen })
		},
		mutationFn: ({ instanceId, target, frozen }: FreezeVariables) =>
			set_project_locked(instanceId, target.originalFilePath, frozen),
		onError: (error, { instanceId, target, frozen }) => {
			updateCachedItems(instanceId, target, { locked: !frozen })
			handleError(error)
		},
		onSettled: (_data, _error, { instanceId }) => markContentStale(instanceId),
	})

	function updateContentList(instanceId: string, update: (items: ContentItem[]) => ContentItem[]) {
		queryClient.setQueryData<InstanceContentData>(instanceKeys.content(instanceId), (current) =>
			current?.contentItems ? { ...current, contentItems: update(current.contentItems) } : current,
		)
	}

	function beginTargets(items: ContentItem[], blocksList: boolean) {
		return items.flatMap((mod) => {
			if (!mod.file_path || !canMutateContent(mod)) return []
			const operation = operations.begin(mod, blocksList)
			return operation
				? [
						{
							mod,
							originalFileName: mod.file_name,
							originalFilePath: mod.file_path,
							operation,
						},
					]
				: []
		})
	}

	async function setEnabled(items: ContentItem[], enabled: boolean, reconcile = true) {
		const targets = beginTargets(items, false).map((target) => ({
			...target,
			previousEnabled: target.mod.enabled,
		}))
		if (targets.length === 0) return
		try {
			await setEnabledMutation.mutateAsync({ instanceId: instance.value.id, enabled, targets })
			if (reconcile) await reconcileSharedInstancePublishState()
		} catch {
			return
		} finally {
			for (const { operation } of targets) operations.finish(operation)
		}
	}

	async function remove(items: ContentItem[]) {
		const targets = beginTargets(items, true)
		if (targets.length === 0) return
		try {
			await removeMutation.mutateAsync({ instanceId: instance.value.id, targets })
		} catch {
			return
		} finally {
			for (const { operation } of targets) operations.finish(operation)
		}
	}

	async function setFrozen(item: ContentItem, frozen: boolean) {
		const [target] = beginTargets([item], true)
		if (!target) return
		try {
			await freezeMutation.mutateAsync({ instanceId: instance.value.id, target, frozen })
		} catch {
			return
		} finally {
			operations.finish(target.operation)
		}
	}

	return { setEnabled, remove, setFrozen }
}
