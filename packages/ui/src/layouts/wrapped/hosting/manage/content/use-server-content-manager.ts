import type { Archon } from '@modrinth/api-client'
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useVIntl } from '#ui/composables/i18n'
import type { ContentManagerContext } from '#ui/layouts/shared/content-tab/providers/content-manager'
import type { ContentItem, ContentOwner, ContentType } from '#ui/layouts/shared/content-tab/types'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectServerSettingsModal,
} from '#ui/providers'

import { injectServerPlayMigration } from '../providers/server-play-migration'
import {
	addonToContentItem,
	contentTypesForLoader,
	type FileInstallProgressItem,
	primaryContentType,
	withInstallProgress,
} from './content-items'
import { useContentBusyState } from './use-content-busy-state'
import { useContentGroups } from './use-content-groups'
import { useContentMutations } from './use-content-mutations'
import { useContentQueries } from './use-content-queries'
import { useContentUpload } from './use-content-upload'
import { useManagedModpack } from './use-managed-modpack'
import { useStoredInstalls } from './use-stored-installs'

export function useServerContentManager() {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const { server, worldId, installProgressItems } = injectModrinthServerContext()
	const { browseServerContent } = injectServerSettingsModal()
	const serverPlayMigration = injectServerPlayMigration(null)
	const router = useRouter()
	const serverId = useRoute().params.id as string

	const busy = useContentBusyState()
	const migrationRequired = computed(() => !!serverPlayMigration?.required.value)
	const queries = useContentQueries(serverId)
	const { contentQuery, contentProjectsById, getAddonForItem } = queries
	useStoredInstalls(serverId, contentQuery.data)

	function toContentItem(addon: Archon.Content.v1.Addon) {
		return addonToContentItem(addon, {
			client,
			worldId: worldId.value,
			project: addon.project_id ? contentProjectsById.value.get(addon.project_id) : undefined,
			migrationRequired: migrationRequired.value,
			formatMessage,
		})
	}

	const fileInstallProgressItems = computed(() =>
		installProgressItems.value.filter(
			(item): item is FileInstallProgressItem =>
				item.world_id === worldId.value && item.key.type === 'file',
		),
	)

	const items = computed<ContentItem[]>(() =>
		(contentQuery.data.value?.addons ?? []).map((addon) =>
			withInstallProgress(toContentItem(addon), fileInstallProgressItems.value),
		),
	)

	const contentReadyPending = computed(
		() =>
			contentQuery.isLoading.value &&
			contentQuery.data.value === undefined &&
			items.value.length === 0,
	)

	const primaryType = computed(() => primaryContentType(server.value?.loader))
	const contentTypes = computed(() => contentTypesForLoader(server.value?.loader))

	const mutations = useContentMutations({
		serverId,
		queries,
		items,
		actionDisabled: busy.actionDisabled,
		migrationRequired,
	})

	const upload = useContentUpload({ primaryType, actionDisabled: busy.actionDisabled })

	const modpack = useManagedModpack({
		serverId,
		queries,
		busy,
		toContentItem,
		setEnabled: mutations.setEnabled,
		setEnabledFor: mutations.setEnabledFor,
	})

	function browse(contentType: ContentType = primaryType.value) {
		if (busy.actionDisabled.value) return
		if (browseServerContent) {
			browseServerContent({ serverId, worldId: worldId.value, type: contentType })
			return
		}

		router.push({
			path: `/discover/${contentType}s`,
			query: { sid: serverId, wid: worldId.value },
		})
	}

	function hasModrinthProject(item: ContentItem) {
		return !!getAddonForItem(item)?.project_id || (!!item.installing && !!item.project?.id)
	}

	function projectPath(item: ContentItem) {
		return `/${item.project_type}/${item.project?.slug ?? item.project?.id}`
	}

	const context: ContentManagerContext = {
		items,
		loading: computed(() => contentQuery.isLoading.value),
		error: computed(() => contentQuery.error.value ?? null),
		refresh: async () => {
			await contentQuery.refetch()
		},
		types: contentTypes,
		target: computed(() => ({
			kind: 'server' as const,
			gameVersion: contentQuery.data.value?.game_version ?? server.value?.mc_version ?? '',
			loader: contentQuery.data.value?.modloader ?? server.value?.loader ?? '',
		})),
		busy: computed(() =>
			busy.actionDisabled.value ? { message: busy.contentBusyMessage.value ?? undefined } : null,
		),
		permissions: () => ({ toggle: true, remove: true, changeVersion: true }),
		canAdd: busy.canSetup,
		addDeniedReason: busy.permissionDeniedMessage.value,
		setEnabled: mutations.setEnabled,
		remove: mutations.remove,
		setEnabledFor: mutations.setEnabledFor,
		installVersion: mutations.installVersion,
		applyUpdates: mutations.applyUpdates,
		browse,
		upload: upload.upload,
		managed: modpack.managedContent,
		links: {
			project: (item) => (hasModrinthProject(item) ? projectPath(item) : undefined),
			version: (item) =>
				hasModrinthProject(item) && item.version?.id
					? `${projectPath(item)}/version/${item.version.id}`
					: undefined,
			owner: (owner: ContentOwner) => owner.link ?? `/${owner.type}/${owner.id}`,
		},
		groups: useContentGroups({ serverId, queries, items }),
		persistKey: `server:${serverId}:${worldId.value}`,
		disableWhileMutating: false,
	}

	return {
		context,
		contentReadyPending,
		busy,
		upload,
		modpack,
		environmentWarningModal: mutations.environmentWarningModal,
	}
}
