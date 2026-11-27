import type { Archon, Labrinth } from '@modrinth/api-client'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import { switchCurseForgeModpackFile } from '#ui/components/flows/creation-flow-modal/curseforge'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import type ManagedContentModal from '#ui/layouts/shared/content-tab/components/managed-content-modal/index.vue'
import { contentVersionQueryOptions } from '#ui/layouts/shared/content-tab/queries'
import type {
	ContentItem,
	ContentSide,
	ManagedContent,
	ManagedContentUpdate,
} from '#ui/layouts/shared/content-tab/types'
import { summarizeManagedContent } from '#ui/layouts/shared/content-tab/utils/managed-content'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	injectServerSettingsModal,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentBusyState } from './use-content-busy-state'
import type { ContentQueries } from './use-content-queries'
import { useCurseForgeModpackFiles } from './use-curseforge-modpack-files'

const messages = defineMessages({
	failedToUnlink: {
		id: 'hosting.content.failed-to-unlink',
		defaultMessage: 'Failed to unlink modpack',
	},
	failedToLoadModpackContent: {
		id: 'hosting.content.failed-to-load-modpack-content',
		defaultMessage: 'Failed to load modpack content',
	},
	failedToUpdate: {
		id: 'hosting.content.failed-to-update',
		defaultMessage: 'Failed to update',
	},
})

function publishedTime(version: Pick<Labrinth.Versions.v2.Version, 'date_published'>) {
	return new Date(version.date_published).getTime()
}

export interface ManagedModpackOptions {
	serverId: string
	queries: ContentQueries
	busy: ContentBusyState
	toContentItem: (addon: Archon.Content.v1.Addon) => ContentItem
	setEnabled: (items: ContentItem[], enabled: boolean) => Promise<void>
	setEnabledFor: (item: ContentItem, side: ContentSide, enabled: boolean) => Promise<void>
}

export function useManagedModpack({
	serverId,
	queries,
	busy,
	toContentItem,
	setEnabled,
	setEnabledFor,
}: ManagedModpackOptions) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { addNotification } = injectNotificationManager()
	const { openServerSettings } = injectServerSettingsModal()
	const { worldId, beginInstallation } = injectModrinthServerContext()
	const { contentQuery, modpackContentQuery, modpackAddons, prefetchModpackContent } = queries
	const { actionDisabled, isInstallingContent, setupBusyMessage } = busy

	const modpackContentModal = ref<InstanceType<typeof ManagedContentModal>>()
	const isModpackContentModalOpen = ref(false)

	function notifyError(title: string, error: unknown) {
		addNotification({
			type: 'error',
			title,
			text: error instanceof Error ? error.message : undefined,
		})
	}

	const modpack = computed(() => contentQuery.data.value?.modpack ?? null)

	const modpackProjectId = computed(() => {
		const spec = modpack.value?.spec
		return spec?.platform === 'modrinth' ? spec.project_id : null
	})

	const modpackVersionsQuery = useQuery(
		computed(() => ({
			...contentVersionQueryOptions.projectVersions(modpackProjectId.value ?? '', client),
			enabled: !!modpackProjectId.value,
		})),
	)

	const projectQuery = useQuery({
		queryKey: computed(() => ['labrinth', 'project', modpackProjectId.value]),
		queryFn: () => client.labrinth.projects_v2.get(modpackProjectId.value!),
		enabled: computed(() => !!modpackProjectId.value),
	})

	const curseforgeLink = computed(() => {
		const spec = modpack.value?.spec
		return spec?.platform === 'curseforge' ? (spec.link ?? null) : null
	})

	const curseforge = useCurseForgeModpackFiles({ client, worldId, link: curseforgeLink })

	const modpackVersions = computed(() =>
		curseforgeLink.value ? curseforge.versions.value : (modpackVersionsQuery.data.value ?? []),
	)

	const currentVersionId = computed(() => {
		const spec = modpack.value?.spec
		if (spec?.platform === 'modrinth') return spec.version_id
		return curseforgeLink.value ? String(curseforgeLink.value.file_id) : null
	})

	const newestUpdateVersion = computed(() => {
		const currentId = currentVersionId.value
		if (!currentId) return null

		const versions = [...modpackVersions.value].sort((a, b) => publishedTime(b) - publishedTime(a))
		const installedPublishedAt = modpack.value?.date_published
		const storedTime = installedPublishedAt
			? publishedTime({ date_published: installedPublishedAt })
			: NaN
		const currentVersion = versions.find((version) => version.id === currentId)
		const currentTime = Number.isNaN(storedTime)
			? currentVersion
				? publishedTime(currentVersion)
				: NaN
			: storedTime

		return (
			versions.find((version) => {
				if (version.id === currentId) return false
				if (Number.isNaN(currentTime)) return true
				return publishedTime(version) > currentTime
			}) ?? null
		)
	})

	async function installVersion(version: Labrinth.Versions.v2.Version) {
		if (actionDisabled.value) return
		const spec = modpack.value?.spec
		try {
			if (curseforgeLink.value) {
				const file = await curseforge.findFile(version.id)
				if (!file) return
				beginInstallation(
					await switchCurseForgeModpackFile(client, worldId.value!, curseforgeLink.value, file),
				)
			} else if (spec?.platform === 'modrinth') {
				await client.archon.content_v1.installContent(serverId, worldId.value!, {
					content_variant: 'modpack',
					spec: {
						platform: 'modrinth',
						project_id: spec.project_id,
						version_id: version.id,
					},
					soft_override: true,
				})
			}
		} catch (err) {
			notifyError(formatMessage(messages.failedToUpdate), err)
		}
	}

	const update = computed<ManagedContentUpdate | undefined>(() => {
		const mp = modpack.value
		if (!mp) return undefined
		const projectId = modpackProjectId.value
		if (curseforgeLink.value) {
			return {
				currentVersionId: currentVersionId.value,
				initialVersionId: newestUpdateVersion.value?.id ?? mp.has_update ?? undefined,
				external: true,
				fetchVersions: curseforge.fetchVersions,
				install: installVersion,
			}
		}
		if (!projectId) return undefined
		return {
			currentVersionId: currentVersionId.value,
			initialVersionId: newestUpdateVersion.value?.id ?? mp.has_update ?? undefined,
			fetchVersions: () =>
				queryClient.fetchQuery(contentVersionQueryOptions.projectVersions(projectId, client)),
			install: installVersion,
		}
	})

	function showModpackContent(addons: Archon.Content.v1.Addon[]) {
		modpackAddons.value = addons
		modpackContentModal.value?.show(addons.map(toContentItem))
	}

	async function viewContent() {
		isModpackContentModalOpen.value = true

		if (modpackContentQuery.data.value) {
			showModpackContent(modpackContentQuery.data.value.addons ?? [])
			return
		}

		modpackContentModal.value?.showLoading()
		try {
			const { data } = await modpackContentQuery.refetch()
			if (!data) throw new Error('Failed to load modpack content')
			showModpackContent(data.addons ?? [])
		} catch (err) {
			isModpackContentModalOpen.value = false
			modpackContentModal.value?.hide()
			notifyError(formatMessage(messages.failedToLoadModpackContent), err)
		}
	}

	watch(
		() => modpackContentQuery.data.value?.addons,
		(addons) => {
			if (!isModpackContentModalOpen.value || !addons) return
			modpackAddons.value = addons
			modpackContentModal.value?.setItems(addons.map(toContentItem))
		},
	)

	async function unlink() {
		if (actionDisabled.value) return
		try {
			await client.archon.content_v1.unlinkModpack(serverId, worldId.value!)
		} catch (err) {
			notifyError(formatMessage(messages.failedToUnlink), err)
		}
	}

	const managedContent = computed<ManagedContent | null>(() => {
		const mp = modpack.value
		if (!mp) return null
		const spec = mp.spec
		const isLocal = spec.platform === 'local_file'
		const project = projectQuery.data.value
		const projectId = modpackProjectId.value
		const addons = modpackContentQuery.data.value?.addons
		const summary = addons
			? summarizeManagedContent(addons.map(toContentItem))
			: modpackContentQuery.isLoading.value
				? undefined
				: []
		const title =
			spec.platform === 'modrinth'
				? (project?.title ?? mp.title ?? spec.project_id)
				: (mp.title ?? (spec.platform === 'local_file' ? spec.name : spec.filename))
		return {
			card: {
				kind: 'modpack',
				installing: isInstallingContent.value,
				manager: {
					name: title,
					iconUrl: (projectId ? (project?.icon_url ?? mp.icon_url) : mp.icon_url) ?? undefined,
					link: projectId
						? `/project/${project?.slug ?? projectId}`
						: curseforgeLink.value
							? `https://www.curseforge.com/minecraft/modpacks/${curseforgeLink.value.slug}`
							: undefined,
				},
				summary,
				versionNumber: isLocal ? undefined : (mp.version_number ?? undefined),
				versionLink:
					spec.platform === 'modrinth'
						? `/project/${project?.slug ?? spec.project_id}/version/${spec.version_id}`
						: undefined,
				updatedAt: isLocal ? undefined : (mp.date_published ?? undefined),
				canSwitchVersion: !!update.value,
			},
			locked: false,
			disabledReason: actionDisabled.value
				? (setupBusyMessage.value ?? formatMessage(commonMessages.installingLabel))
				: undefined,
			view: () => void viewContent(),
			update: update.value,
			openSettings: () => openServerSettings({ tabId: 'installation' }),
			unlink: () => void unlink(),
			prefetchView: prefetchModpackContent,
			prefetchPrimaryAction: () => {
				if (!projectId) return
				void queryClient.prefetchQuery(
					contentVersionQueryOptions.projectVersions(projectId, client),
				)
			},
		}
	})

	function toggleItem(item: ContentItem, enabled: boolean) {
		void setEnabled([item], enabled).catch(() => undefined)
	}

	function setItemEnabledFor(item: ContentItem, side: ContentSide, enabled: boolean) {
		void setEnabledFor(item, side, enabled).catch(() => undefined)
	}

	return {
		managedContent,
		modpackContentModal,
		isModpackContentModalOpen,
		toggleItem,
		setItemEnabledFor,
	}
}
