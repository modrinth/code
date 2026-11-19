import type { Labrinth } from '@modrinth/api-client'
import {
	commonMessages,
	type ConfirmDisableModal,
	type ContentItem,
	contentVersionQueryOptions,
	dedupeManagedContentItems,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	type ManagedContent,
	type ManagedContentModal,
	type ManagedContentModalState,
	type ManagedContentProject,
	type ManagedContentUpdate,
	type ManagedContentVersion,
	summarizeManagedContent,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, onUnmounted, type Ref, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import type { useManagedContentPolicy } from '@/composables/instances/use-managed-content-policy'
import { edit, get_linked_modpack_content, getInstanceIconUrl } from '@/helpers/instance'
import type { InstanceContentData } from '@/helpers/instance-content'
import type { CacheBehaviour, GameInstance } from '@/helpers/types'
import type { injectInstancePage } from '@/pages/instance/instance-context'
import { instanceKeys } from '@/pages/instance/query-options'
import type { injectSharedInstance } from '@/pages/instance/shared-instance-context'

const messages = defineMessages({
	modpackContentHeader: {
		id: 'app.instance.content.managed-content.modpack-header',
		defaultMessage: 'Modpack content',
	},
	sharedContentHeader: {
		id: 'app.instance.content.managed-content.shared-header',
		defaultMessage: 'Shared content',
	},
})

let savedModalState: { instanceId: string; state: ManagedContentModalState } | null = null

export interface ManagedContentOptions {
	instance: Ref<GameInstance>
	instancePage: ReturnType<typeof injectInstancePage>
	sharedInstanceState: ReturnType<typeof injectSharedInstance>
	policy: ReturnType<typeof useManagedContentPolicy>
	contentData: Readonly<Ref<InstanceContentData | undefined>>
	installedItems: Readonly<Ref<ContentItem[]>>
	isInstanceBusy: Readonly<Ref<boolean>>
	isModpackUpdating: Readonly<Ref<boolean>>
	setEnabled: (items: ContentItem[], enabled: boolean, reconcile?: boolean) => Promise<void>
	reconcileSharedInstancePublishState: () => Promise<void>
	refetchContent: (cacheBehaviour?: CacheBehaviour) => Promise<void>
	fetchProjectVersions: (projectId: string) => Promise<Labrinth.Versions.v2.Version[]>
	installModpackVersion: (version: Labrinth.Versions.v2.Version) => Promise<void>
	managedContentModal: Readonly<Ref<InstanceType<typeof ManagedContentModal> | null | undefined>>
}

export function useManagedContent({
	instance,
	instancePage,
	sharedInstanceState,
	policy,
	contentData,
	installedItems,
	isInstanceBusy,
	isModpackUpdating,
	setEnabled,
	reconcileSharedInstancePublishState,
	refetchContent,
	fetchProjectVersions,
	installModpackVersion,
	managedContentModal,
}: ManagedContentOptions) {
	const { formatMessage } = useVIntl()
	const { handleError } = injectNotificationManager()
	const queryClient = useQueryClient()
	const client = injectModrinthClient()
	const route = useRoute()
	const router = useRouter()

	const managedDisableConfirmModal = ref<InstanceType<typeof ConfirmDisableModal> | null>()
	const pendingManagedDisableItems = ref<ContentItem[]>([])

	const localImportedModpackUnlinked = ref(false)
	watch(
		() => instance.value.link,
		() => {
			localImportedModpackUnlinked.value = false
		},
	)

	const linkedModpackProject = computed<ManagedContentProject | null>(
		() => contentData.value?.modpack?.project ?? null,
	)
	const linkedModpackVersion = computed<ManagedContentVersion | null>(
		() => contentData.value?.modpack?.version ?? null,
	)

	const localImportedModpackProject = computed<ManagedContentProject | null>(() => {
		const link = instance.value.link
		if (localImportedModpackUnlinked.value || link?.type !== 'imported_modpack') return null
		return {
			id: link.filename ?? instance.value.id,
			slug: link.filename ?? instance.value.id,
			title: link.name ?? instance.value.name,
			icon_url: getInstanceIconUrl(instance.value.icon_path) ?? undefined,
			filename: link.filename ?? undefined,
		}
	})

	const displayedModpackProject = computed(
		() => linkedModpackProject.value ?? localImportedModpackProject.value,
	)

	const modpackContentQueryKey = computed(() => instanceKeys.linkedContent(instance.value.id))
	const modpackContentQuery = useQuery({
		queryKey: modpackContentQueryKey,
		networkMode: 'always',
		queryFn: () => get_linked_modpack_content(instance.value.id),
		enabled: computed(
			() =>
				!!instance.value?.id &&
				!!instance.value?.link &&
				instance.value.install_stage === 'installed',
		),
	})

	const hasSharedManagedContent = computed(() => {
		if (instance.value.shared_instance?.role === 'owner') return false
		const linkType = instance.value.link?.type
		return (
			!!instance.value.shared_instance ||
			linkType === 'server_project' ||
			linkType === 'server_project_modpack'
		)
	})

	const managedContentItems = computed(() => {
		const linkedContent = instance.value.link ? (modpackContentQuery.data.value ?? []) : []
		const sourcedContent = hasSharedManagedContent.value
			? installedItems.value.filter((item) =>
					['server_project', 'shared_instance'].includes(item.source_kind ?? ''),
				)
			: []
		return dedupeManagedContentItems([...linkedContent, ...sourcedContent])
	})

	const managedContentSummary = computed(() =>
		modpackContentQuery.isLoading.value && modpackContentQuery.data.value === undefined
			? undefined
			: summarizeManagedContent(managedContentItems.value),
	)

	const isPackLocked = computed(
		() =>
			instance.value.content_locked ||
			instance.value?.link?.type === 'modrinth_modpack' ||
			instance.value?.link?.type === 'server_project_modpack',
	)

	const isSharedMember = computed(() => instance.value.shared_instance?.role === 'member')

	const modpackUpdate = computed<ManagedContentUpdate | undefined>(() => {
		const link = instance.value.link
		if (isSharedMember.value || policy.isContentLocked.value) return undefined
		if (link?.type !== 'modrinth_modpack') return undefined
		const projectId = link.project_id
		return {
			currentVersionId: link.version_id,
			initialVersionId: contentData.value?.modpack?.updateVersionId ?? link.version_id,
			fetchVersions: () => fetchProjectVersions(projectId),
			install: installModpackVersion,
		}
	})

	const managed = computed<ManagedContent | null>(() => {
		const attachment = instance.value.shared_instance
		const sharedManager = sharedInstanceState.manager.value
		const linkedProject = instancePage.linkedProject.value
		const linkType = instance.value.link?.type
		const isSharedOwner = attachment?.role === 'owner'
		const actions = {
			locked: isPackLocked.value,
			view: openManagedContent,
			update: modpackUpdate.value,
			primaryAction: isSharedMember.value ? instancePage.reviewSharedInstanceUpdate : undefined,
			openSettings: () => instancePage.openSettings(1),
			unlink: unpairInstance,
			prefetchView,
			prefetchPrimaryAction,
		}

		if (
			(linkType === 'server_project' || linkType === 'server_project_modpack') &&
			managedContentItems.value.length === 0
		) {
			return null
		}

		if (
			!isSharedOwner &&
			(attachment || linkType === 'server_project' || linkType === 'server_project_modpack')
		) {
			const serverManaged =
				sharedManager?.type === 'server' ||
				!!attachment?.server_manager_name ||
				linkType === 'server_project' ||
				linkType === 'server_project_modpack' ||
				(!attachment && instancePage.isServerInstance.value)
			const managerName = serverManaged
				? (sharedManager?.name ??
					attachment?.server_manager_name ??
					linkedProject?.name ??
					instance.value.name)
				: (sharedManager?.name ?? instance.value.name)
			const managerIcon = serverManaged
				? (sharedManager?.avatarUrl ??
					getInstanceIconUrl(attachment?.server_manager_icon_url) ??
					linkedProject?.icon_url ??
					undefined)
				: (sharedManager?.avatarUrl ?? getInstanceIconUrl(instance.value.icon_path) ?? undefined)
			const managerLink = serverManaged
				? linkedProject
					? {
							path: `/project/${linkedProject.slug ?? linkedProject.id}`,
							query: { i: instancePage.instanceId.value },
						}
					: undefined
				: sharedManager?.type === 'user'
					? `/user/${encodeURIComponent(sharedManager.name)}`
					: undefined
			const disabled = attachment?.status === 'applying' || isInstanceBusy.value

			return {
				...actions,
				card: {
					kind: serverManaged ? 'server' : 'shared-instance',
					installing: isInstanceBusy.value,
					manager: { name: managerName, iconUrl: managerIcon, link: managerLink },
					summary: managedContentSummary.value,
					syncedAt: sharedInstanceState.lastUpdateCheckAt.value,
					updateAvailable: instancePage.sharedInstanceUpdateAvailable.value,
				},
				disabledReason: disabled ? formatMessage(commonMessages.updatingLabel) : undefined,
			}
		}

		const project = displayedModpackProject.value
		if (!project) return null
		const disabled = isModpackUpdating.value || isInstanceBusy.value

		return {
			...actions,
			card: {
				kind: 'modpack',
				installing: isInstanceBusy.value,
				manager: {
					name: project.title,
					iconUrl: project.icon_url ?? undefined,
					link: linkedModpackProject.value
						? {
								path: `/project/${project.slug ?? project.id}`,
								query: { i: instancePage.instanceId.value },
							}
						: undefined,
				},
				summary: managedContentSummary.value,
				versionNumber: linkedModpackVersion.value?.version_number,
				versionLink:
					linkedModpackProject.value && linkedModpackVersion.value
						? {
								path: `/project/${linkedModpackProject.value.slug ?? linkedModpackProject.value.id}/version/${linkedModpackVersion.value.id}`,
								query: { i: instancePage.instanceId.value },
							}
						: undefined,
				updatedAt: linkedModpackVersion.value?.date_published,
			},
			disabledReason: disabled ? formatMessage(commonMessages.updatingLabel) : undefined,
		}
	})

	const managedContentModalHeader = computed(() =>
		formatMessage(
			managed.value?.card.kind === 'modpack'
				? messages.modpackContentHeader
				: messages.sharedContentHeader,
		),
	)

	function prefetchView() {
		if (!instance.value.link || instance.value.install_stage !== 'installed') return
		void queryClient.prefetchQuery({
			queryKey: modpackContentQueryKey.value,
			networkMode: 'always',
			queryFn: () => get_linked_modpack_content(instance.value.id),
		})
	}

	function prefetchPrimaryAction() {
		const link = instance.value.link
		if (link?.type !== 'modrinth_modpack' || !link.project_id) return
		void queryClient.prefetchQuery(
			contentVersionQueryOptions.projectVersions(link.project_id, client),
		)
		const updateVersionId = contentData.value?.modpack?.updateVersionId
		if (updateVersionId) {
			void queryClient.prefetchQuery(contentVersionQueryOptions.version(updateVersionId, client))
		}
	}

	async function openManagedContent() {
		if (!instance.value?.id) return
		if (modpackContentQuery.data.value !== undefined) {
			managedContentModal.value?.show(managedContentItems.value)
			return
		}
		managedContentModal.value?.showLoading()
		const { data, error } = await modpackContentQuery.refetch()
		if (data !== undefined) {
			managedContentModal.value?.show(managedContentItems.value)
		} else {
			if (error) handleError(error)
			managedContentModal.value?.hide()
		}
	}

	async function refreshManagedContentItems(cacheBehaviour?: CacheBehaviour) {
		if (!instance.value?.id) return
		const contentItems = await queryClient
			.fetchQuery({
				queryKey: modpackContentQueryKey.value,
				networkMode: 'always',
				queryFn: () => get_linked_modpack_content(instance.value.id, cacheBehaviour),
			})
			.catch(handleError)
		if (contentItems) managedContentModal.value?.setItems(managedContentItems.value)
	}

	async function setManagedEnabled(items: ContentItem[], enabled: boolean) {
		await setEnabled(items, enabled, false)
		await reconcileSharedInstancePublishState()
	}

	async function toggleManagedItem(item: ContentItem, enabled: boolean) {
		if (!enabled && policy.disableWarning([item])) {
			pendingManagedDisableItems.value = [item]
			managedDisableConfirmModal.value?.show()
			return
		}
		await setEnabled([item], enabled)
	}

	async function toggleManagedItems(items: ContentItem[], enabled: boolean) {
		if (!enabled && policy.disableWarning(items)) {
			pendingManagedDisableItems.value = items
			managedDisableConfirmModal.value?.show()
			return
		}
		await setManagedEnabled(items, enabled)
	}

	async function confirmManagedDisable() {
		const items = [...pendingManagedDisableItems.value]
		pendingManagedDisableItems.value = []
		await setManagedEnabled(items, false)
	}

	async function unpairInstance() {
		await edit(instance.value.id, { link: null as unknown as undefined })
		queryClient.setQueryData<InstanceContentData>(
			instanceKeys.content(instance.value.id),
			(data) => (data ? { ...data, modpack: null } : data),
		)
		localImportedModpackUnlinked.value = true
		await refetchContent()
	}

	function matchesHighlightedFile(item: ContentItem) {
		const path = route.query.highlight
		return (
			typeof path === 'string' && (item.file_path === path || item.file_path === `${path}.disabled`)
		)
	}

	function watchHighlightedManagedItem(visibleItems: Readonly<Ref<ContentItem[]>>) {
		const highlightedManagedItemId = computed(() =>
			visibleItems.value.some(matchesHighlightedFile)
				? undefined
				: managedContentItems.value.find(matchesHighlightedFile)?.id,
		)
		watch(
			[highlightedManagedItemId, managedContentModal],
			([id, modal]) => {
				if (id && modal) modal.show(managedContentItems.value, id)
			},
			{ flush: 'post' },
		)
	}

	async function restoreModalState() {
		const saved = savedModalState
		savedModalState = null
		if (saved?.instanceId !== instance.value.id) return
		await nextTick()
		managedContentModal.value?.restore(saved.state)
	}

	const removeBeforeEach = router.beforeEach(() => {
		const state = managedContentModal.value?.getState()
		savedModalState = state ? { instanceId: instance.value.id, state } : null
	})
	onUnmounted(removeBeforeEach)

	return {
		managedDisableConfirmModal,
		managedContentItems,
		pendingManagedDisableItems,
		managed,
		managedContentModalHeader,
		displayedModpackProject,
		isPackLocked,
		matchesHighlightedFile,
		watchHighlightedManagedItem,
		refreshManagedContentItems,
		toggleManagedItem,
		toggleManagedItems,
		confirmManagedDisable,
		restoreModalState,
	}
}
