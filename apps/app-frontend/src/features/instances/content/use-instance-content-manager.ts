import type { Labrinth } from '@modrinth/api-client'
import { ClipboardCopyIcon, FolderOpenIcon, LockIcon, LockOpenIcon } from '@modrinth/assets'
import {
	type ButtonMenuOption,
	commonMessages,
	type ContentConfirmResult,
	type ContentItem,
	type ContentManagerContext,
	type ContentOwner,
	type ContentType,
	defineMessages,
	injectNotificationManager,
	type ManagedContentModal,
	type UpdateAllSelection,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { useIsMutating, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import type SyncedContentModal from '@/components/ui/instance/SyncedContentModal.vue'
import { useManagedContentPolicy } from '@/composables/instances/use-managed-content-policy'
import { useSyncedPackActions } from '@/composables/instances/use-synced-pack-actions'
import { useAppEvent } from '@/composables/use-app-event'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get_version_many } from '@/helpers/cache.js'
import { install_bulk_update_content } from '@/helpers/install'
import { get_shared_instance_publish_preview } from '@/helpers/instance'
import { loadInstanceContentData } from '@/helpers/instance-content'
import type { CacheBehaviour } from '@/helpers/types'
import { highlightModInInstance } from '@/helpers/utils.js'
import { injectInstancePage } from '@/pages/instance/instance-context'
import { instanceContentQueryOptions, instanceKeys } from '@/pages/instance/query-options'
import { injectSharedInstance } from '@/pages/instance/shared-instance-context'
import { injectContentInstall } from '@/providers/content-install'

import { findDependentItems, normalizeContentItem } from './content-items'
import { contentMutationKey, useContentMutations } from './use-content-mutations'
import { useContentOperations } from './use-content-operations'
import { useContentUpdater } from './use-content-updater'
import { useContentUpload } from './use-content-upload'
import { useManagedContent } from './use-managed-content'

const messages = defineMessages({
	lockedContent: {
		id: 'app.instance.mods.locked-content',
		defaultMessage: 'Content in locked instances cannot be changed.',
	},
	freezeContent: {
		id: 'app.instance.mods.freeze-content',
		defaultMessage: 'Freeze version',
	},
	unfreezeContent: {
		id: 'app.instance.mods.unfreeze-content',
		defaultMessage: 'Unfreeze version',
	},
})

function contentOwnerLink(owner: ContentOwner): NonNullable<ContentOwner['link']> {
	if (owner.type === 'user') return `/user/${encodeURIComponent(owner.id)}`
	return () => {
		void openUrl(`https://modrinth.com/organization/${owner.id}`)
	}
}

export function useInstanceContentManager() {
	const { formatMessage } = useVIntl()
	const { handleError } = injectNotificationManager()
	const { installingItems, installRevisionByInstance, installFailureRevisionByInstance } =
		injectContentInstall()
	const route = useRoute()
	const queryClient = useQueryClient()
	const debug = useDebugLogger('Mods:ContentUpdate')
	const appSettings = useAppSettings()
	const skipNonEssentialWarnings = computed(() => appSettings.skipNonEssentialWarnings)

	const instancePage = injectInstancePage()
	const sharedInstanceState = injectSharedInstance()
	const instance = instancePage.instance
	const policy = useManagedContentPolicy(computed(() => instance.value))
	const {
		isManagedModpack: isSharedMember,
		isContentLocked,
		canMutateContent,
		canUpdateContent: canUpdateProject,
	} = policy

	const isInstanceBusy = computed(() => instance.value?.install_stage !== 'installed')
	const showSharedContentFilter = computed(() => instance.value.shared_instance?.role === 'member')

	const syncedContentModal = ref<InstanceType<typeof SyncedContentModal>>()
	const managedContentModal = ref<InstanceType<typeof ManagedContentModal> | null>()

	const contentQuery = useQuery(
		computed(() => ({
			...instanceContentQueryOptions(instancePage.instanceId.value),
			enabled: !!instancePage.instanceId.value,
		})),
	)
	watch(contentQuery.error, (error) => {
		if (error) handleError(error)
	})

	const contentData = computed(() =>
		contentQuery.data.value?.path === instance.value.id ? contentQuery.data.value : undefined,
	)
	const loading = computed(() => contentData.value === undefined && !contentQuery.error.value)

	async function refetchContent(cacheBehaviour?: CacheBehaviour, staleTime = 0) {
		if (!instance.value) return
		await queryClient.fetchQuery({
			...instanceContentQueryOptions(instance.value.id),
			queryFn: () => loadInstanceContentData(instance.value.id, cacheBehaviour, handleError),
			staleTime,
		})
	}

	async function refreshContentState(cacheBehaviour?: CacheBehaviour) {
		await refetchContent(cacheBehaviour)
		await managedState.refreshManagedContentItems(cacheBehaviour)
	}

	async function reconcileSharedInstancePublishState() {
		if (instance.value.shared_instance?.role !== 'owner') return
		await get_shared_instance_publish_preview(instance.value.id).catch((error) => {
			debug('Failed to reconcile shared instance publish state', { error })
		})
	}

	const operations = useContentOperations((fileName, state) =>
		managedContentModal.value?.updateItem(fileName, state),
	)

	const packActions = useSyncedPackActions(instance, syncedContentModal, canMutateContent, () =>
		refreshContentState('must_revalidate'),
	)

	const mutations = useContentMutations({
		instance,
		operations,
		canMutateContent,
		managedContentModal,
		deleteSyncedItem: packActions.deleteSyncedItem,
		refreshContentState,
		reconcileSharedInstancePublishState,
	})

	function canChangeContentVersion(item: ContentItem) {
		return canMutateContent(item) && !item.locked
	}

	const updater = useContentUpdater({
		instance,
		canUpdateProject,
		canChangeContentVersion,
		operations,
		refetchContent,
		refreshContentState,
	})

	const installedItems = computed<ContentItem[]>(() =>
		(contentData.value?.contentItems ?? []).map((item) => ({
			...normalizeContentItem(item),
			has_update: canUpdateProject(item),
		})),
	)

	const managedState = useManagedContent({
		instance,
		instancePage,
		sharedInstanceState,
		policy,
		contentData,
		installedItems,
		isInstanceBusy,
		isModpackUpdating: updater.isModpackUpdating,
		setEnabled: mutations.setEnabled,
		reconcileSharedInstancePublishState,
		refetchContent,
		fetchProjectVersions: updater.fetchProjectVersions,
		installModpackVersion: updater.installModpackVersion,
		managedContentModal,
	})

	const installingBuffer = ref<ContentItem[]>([])
	watch(
		() => installingItems.value.get(instance.value.id),
		(items) => {
			if (items && items.length > 0) installingBuffer.value = [...items]
		},
		{ immediate: true, deep: true },
	)
	watch(installedItems, (items) => {
		if (installingBuffer.value.length === 0) return
		const projectIds = new Set(items.map((item) => item.project?.id).filter(Boolean))
		if (installingBuffer.value.every((item) => projectIds.has(item.project?.id))) {
			installingBuffer.value = []
		}
	})
	watch(
		() => installFailureRevisionByInstance.value.get(instance.value.id) ?? 0,
		(revision, previousRevision) => {
			if (revision !== previousRevision) installingBuffer.value = []
		},
	)

	const items = computed<ContentItem[]>(() => {
		const managedFiles = new Set(
			managedState.managedContentItems.value.map((item) => item.file_name),
		)
		const additional = installedItems.value
			.filter((item) => !managedFiles.has(item.file_name))
			.map((item) => (operations.isInstalling(item) ? { ...item, installing: true } : item))
		const pending = (installingItems.value.get(instance.value.id) ?? installingBuffer.value)
			.filter((item) => !managedFiles.has(item.file_name))
			.map(normalizeContentItem)
		if (pending.length === 0) return additional

		const pendingProjectIds = new Set(pending.map((item) => item.project?.id).filter(Boolean))
		const displayed = additional.map((item) =>
			item.project?.id && pendingProjectIds.has(item.project.id)
				? { ...item, installing: true }
				: item,
		)
		const listedProjectIds = new Set(displayed.map((item) => item.project?.id).filter(Boolean))
		const placeholders = pending.filter((item) => !listedProjectIds.has(item.project?.id))
		return placeholders.length > 0 ? [...displayed, ...placeholders] : displayed
	})

	const highlightedItemId = computed(() => {
		const path = route.query.highlight
		if (typeof path !== 'string') return undefined
		return items.value.find(managedState.matchesHighlightedFile)?.id ?? path
	})
	managedState.watchHighlightedManagedItem(items)

	const upload = useContentUpload({ instance, items, isInstanceBusy, refetchContent })

	async function findDependents(targets: ContentItem[]) {
		if (instancePage.isServerInstance.value) return null
		const deletingIds = new Set(targets.map((item) => item.id))
		const remainingItems = installedItems.value.filter((item) => !deletingIds.has(item.id))
		const versionIds = [
			...new Set(remainingItems.map((item) => item.version?.id).filter((id): id is string => !!id)),
		]
		if (versionIds.length === 0) return null
		const versions = (await get_version_many(versionIds).catch((error) => {
			handleError(error as Error)
			return null
		})) as Labrinth.Versions.v2.Version[] | null
		if (!versions) return null
		const dependents = findDependentItems(targets, remainingItems, versions)
		return dependents.length > 0 ? { items: targets, dependents } : null
	}

	const bulkUpdateMutation = useMutation({
		mutationFn: ({
			instanceId,
			updates,
		}: {
			instanceId: string
			updates: Parameters<typeof install_bulk_update_content>[1]
		}) => install_bulk_update_content(instanceId, updates),
		onError: (error) => handleError(error),
	})

	async function applyUpdates(selections: UpdateAllSelection[]) {
		if (isInstanceBusy.value || bulkUpdateMutation.isPending.value) return
		const updates = selections.flatMap((selection) => {
			const item =
				installedItems.value.find((candidate) => candidate.id === selection.id) ??
				installedItems.value.find((candidate) => candidate.project?.id === selection.projectId)
			if (
				!item ||
				!canChangeContentVersion(item) ||
				!item.file_path ||
				item.version?.id === selection.version.id
			)
				return []
			return [{ project_path: item.file_path, version_id: selection.version.id }]
		})
		if (updates.length === 0) return
		await bulkUpdateMutation.mutateAsync({ instanceId: instance.value.id, updates })
	}

	async function confirm(
		action: 'enable' | 'disable' | 'remove',
		targets: ContentItem[],
	): Promise<ContentConfirmResult> {
		if (action === 'remove') {
			const confirmed = await packActions.confirmDeleteItems(targets)
			if (confirmed === undefined) return 'unhandled'
			return confirmed ? 'confirmed' : 'cancelled'
		}
		if (!targets.some((item) => item.synced_pack)) return 'unhandled'
		return (await packActions.confirmAction(action, targets)) ? 'confirmed' : 'cancelled'
	}

	function menu(item: ContentItem): ButtonMenuOption[] {
		const options: ButtonMenuOption[] = packActions.overflowOptions(item)
		options.push({
			id: 'show-file',
			label: formatMessage(commonMessages.showFileButton),
			icon: FolderOpenIcon,
			action: () => highlightModInInstance(instance.value.id, item.file_path),
		})
		if (item.project?.slug) {
			options.push({
				id: 'copy-link',
				label: formatMessage(commonMessages.copyLinkButton),
				icon: ClipboardCopyIcon,
				action: async () => {
					await navigator.clipboard.writeText(
						`https://modrinth.com/${item.project_type}/${item.project?.slug}`,
					)
				},
			})
		}
		if (canMutateContent(item)) {
			options.push(
				{ type: 'divider' },
				{
					id: item.locked ? 'unfreeze-content' : 'freeze-content',
					label: formatMessage(item.locked ? messages.unfreezeContent : messages.freezeContent),
					icon: item.locked ? LockOpenIcon : LockIcon,
					action: () => mutations.setFrozen(item, !item.locked),
				},
			)
		}
		return options
	}

	const contentTypes = computed<ContentType[]>(() =>
		instance.value.loader === 'vanilla'
			? ['resourcepack', 'datapack']
			: ['mod', 'resourcepack', 'datapack', 'shader'],
	)

	const context: ContentManagerContext = {
		items,
		loading,
		error: ref(null),
		refresh: () => refetchContent('must_revalidate'),
		types: contentTypes,
		target: computed(() => ({
			kind: 'instance' as const,
			gameVersion: instance.value.game_version,
			loader: instance.value.loader,
		})),
		busy: computed(() => (isInstanceBusy.value || packActions.isPending.value ? {} : null)),
		permissions: (item) => ({
			toggle: canMutateContent(item),
			remove: canMutateContent(item),
			changeVersion: canChangeContentVersion(item),
		}),
		canAdd: computed(() => !isContentLocked.value),
		addDeniedReason: formatMessage(messages.lockedContent),
		warn: (action, targets) =>
			action === 'remove' ? policy.deleteWarning(targets) : policy.disableWarning(targets),
		confirm,
		skipOptionalConfirms: skipNonEssentialWarnings,
		setEnabled: (targets, enabled) => mutations.setEnabled(targets, enabled),
		remove: mutations.remove,
		findDependents,
		installVersion: updater.installVersion,
		fetchProjectVersions: updater.fetchProjectVersions,
		fetchVersion: updater.fetchVersion,
		applyUpdates,
		updatesRunInBackground: true,
		browse: (type) => {
			if (!instance.value || instance.value.content_locked) return
			void instancePage.browseContent(type ?? contentTypes.value[0])
		},
		upload: () => void upload.upload(),
		managed: managedState.managed,
		links: {
			project: (item) =>
				item.project?.id
					? { path: `/project/${item.project.id}`, query: { i: instancePage.instanceId.value } }
					: undefined,
			version: (item) =>
				item.project?.id && item.version?.id
					? {
							path: `/project/${item.project.id}/version/${item.version.id}`,
							query: { i: instancePage.instanceId.value },
						}
					: undefined,
			owner: contentOwnerLink,
		},
		menu,
		persistKey: instance.value.id,
		searchQueryParam: 'q',
		showSharedContentFilter,
	}

	const handledInstallRevision = ref(0)

	function loadInitialContent() {
		const installRevision = installRevisionByInstance.value.get(instance.value.id) ?? 0
		if (installRevision > handledInstallRevision.value) {
			handledInstallRevision.value = installRevision
			return refetchContent('must_revalidate')
		}
		return refetchContent(undefined, 30_000)
	}

	void loadInitialContent().then(managedState.restoreModalState).catch(handleError)

	watch(
		() => installRevisionByInstance.value.get(instance.value.id) ?? 0,
		async (revision) => {
			if (revision <= handledInstallRevision.value) return
			handledInstallRevision.value = revision
			await refreshContentState('must_revalidate')
		},
	)

	const pendingContentMutations = useIsMutating({ mutationKey: contentMutationKey })

	useAppEvent('instance', async (event) => {
		if (
			instance.value &&
			event.instance_id === instance.value.id &&
			event.event === 'synced' &&
			instance.value.install_stage === 'installed' &&
			pendingContentMutations.value === 0 &&
			!operations.hasActive.value
		) {
			await queryClient.invalidateQueries({ queryKey: instanceKeys.content(instance.value.id) })
		}
	})

	watch(
		() => instance.value?.install_stage,
		async (newStage, oldStage) => {
			if (oldStage !== 'installed' && newStage === 'installed') {
				await refreshContentState('must_revalidate')
			} else if (oldStage === 'not_installed' && newStage === 'pack_installing') {
				await refetchContent()
			}
		},
	)

	watch(
		() => instance.value?.link,
		async (newLink, oldLink) => {
			if (oldLink && !newLink) await refetchContent('must_revalidate')
		},
	)

	watch(
		() => instance.value?.update_channel,
		async (newValue, oldValue) => {
			if (newValue !== oldValue) await refetchContent('must_revalidate')
		},
	)

	return {
		context,
		instance,
		installedItems,
		loading,
		highlightedItemId,
		isInstanceBusy,
		isContentLocked,
		isSharedMember,
		isServerInstance: instancePage.isServerInstance,
		isBlocking: operations.isBlocking,
		policy,
		syncedContentModal,
		managedContentModal,
		canChangeContentVersion,
		managedState,
		upload,
		menu,
	}
}
