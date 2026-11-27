import type { Labrinth } from '@modrinth/api-client'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref, shallowRef } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient } from '#ui/providers/api-client'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { commonMessages } from '#ui/utils/common-messages'
import { versionChangesGameVersion } from '#ui/utils/version-compatibility'

import type { ContentManagerContext } from '../../providers/content-manager'
import { contentVersionQueryOptions } from '../../queries'
import type { ContentItem, ManagedContentUpdate } from '../../types'
import type ConfirmModpackUpdateModal from '../modals/ConfirmModpackUpdateModal.vue'
import type ContentUpdaterModal from '../modals/content-updater-modal/index.vue'

type Version = Labrinth.Versions.v2.Version

type UpdateTarget =
	| { kind: 'item'; item: ContentItem }
	| { kind: 'modpack'; update: ManagedContentUpdate }

const messages = defineMessages({
	failedToLoadVersions: {
		id: 'content.update-flow.failed-to-load-versions',
		defaultMessage: 'Failed to load versions',
	},
	failedToLoadChangelog: {
		id: 'content.update-flow.failed-to-load-changelog',
		defaultMessage: 'Failed to load changelog',
	},
})

function sortByPublished(versions: Version[]) {
	return [...versions].sort((a, b) => Date.parse(b.date_published) - Date.parse(a.date_published))
}

function mergeVersion(versions: Version[], version: Version) {
	return sortByPublished([...versions.filter((candidate) => candidate.id !== version.id), version])
}

export function useUpdateFlow(ctx: ContentManagerContext) {
	const { formatMessage } = useVIntl()
	const queryClient = useQueryClient()
	const client = injectModrinthClient()
	const { addNotification } = injectNotificationManager()
	const debug = useDebugLogger('ContentUpdateFlow')

	const updaterModal = ref<InstanceType<typeof ContentUpdaterModal>>()
	const confirmModpackUpdateModal = ref<InstanceType<typeof ConfirmModpackUpdateModal>>()
	const target = shallowRef<UpdateTarget | null>(null)
	const versions = ref<Version[]>([])
	const loadingVersions = ref(false)
	const loadingChangelog = ref(false)
	const pendingModpackVersion = ref<Version | null>(null)
	const isDowngrade = ref(false)
	let lastRequestId = 0
	let activeRequestId = 0

	const updatingItem = computed(() => (target.value?.kind === 'item' ? target.value.item : null))
	const updatingModpack = computed(() =>
		target.value?.kind === 'modpack' ? target.value.update : null,
	)
	const isExternal = computed(() => !!updatingModpack.value?.external)

	const currentVersionId = computed(
		() => updatingModpack.value?.currentVersionId ?? updatingItem.value?.version?.id ?? '',
	)
	const projectType = computed(() =>
		updatingModpack.value ? 'modpack' : updatingItem.value?.project_type,
	)
	const projectName = computed(() =>
		updatingModpack.value
			? (ctx.managed.value?.card.manager.name ?? formatMessage(commonMessages.modpackLabel))
			: (updatingItem.value?.project?.title ?? updatingItem.value?.file_name),
	)
	const projectIconUrl = computed(() =>
		updatingModpack.value
			? ctx.managed.value?.card.manager.iconUrl
			: (updatingItem.value?.project?.icon_url ?? undefined),
	)
	const actionDisabled = computed(() =>
		updatingModpack.value ? !!ctx.managed.value?.disabledReason : !!ctx.busy.value,
	)
	const actionDisabledTooltip = computed(() =>
		updatingModpack.value ? ctx.managed.value?.disabledReason : ctx.busy.value?.message,
	)
	const backupTip = computed(() =>
		[ctx.managed.value?.card.manager.name, pendingModpackVersion.value?.version_number]
			.filter(Boolean)
			.join(' '),
	)

	function notifyError(title: string, error: unknown) {
		addNotification({
			type: 'error',
			title,
			text: error instanceof Error ? error.message : undefined,
		})
	}

	function fetchVersion(versionId: string) {
		return (
			ctx.fetchVersion?.(versionId) ??
			queryClient.fetchQuery(contentVersionQueryOptions.version(versionId, client))
		)
	}

	function fetchProjectVersions(projectId: string) {
		return (
			ctx.fetchProjectVersions?.(projectId) ??
			queryClient.fetchQuery(contentVersionQueryOptions.projectVersions(projectId, client))
		)
	}

	function isActive(requestId: number) {
		return activeRequestId === requestId
	}

	async function open(
		next: UpdateTarget,
		initialVersionId: string | undefined,
		loadVersions: () => Promise<Version[]>,
		switchMode = false,
	) {
		const requestId = ++lastRequestId
		const external = next.kind === 'modpack' && !!next.update.external
		activeRequestId = requestId
		target.value = next
		versions.value = []
		loadingVersions.value = true
		loadingChangelog.value = false
		pendingModpackVersion.value = null

		await nextTick()
		if (!isActive(requestId)) return
		debug('opening updater', { kind: next.kind, initialVersionId, switchMode })
		updaterModal.value?.show(initialVersionId, { switchMode })

		let loaded: Version[] = []
		try {
			loaded = sortByPublished(await loadVersions())
		} catch (error) {
			if (isActive(requestId)) notifyError(formatMessage(messages.failedToLoadVersions), error)
		}
		if (
			initialVersionId &&
			!external &&
			!loaded.some((version) => version.id === initialVersionId)
		) {
			const pinned = await fetchVersion(initialVersionId).catch(() => null)
			if (pinned) loaded = mergeVersion(loaded, pinned)
		}
		if (!isActive(requestId)) return

		debug('loaded versions', {
			kind: next.kind,
			count: loaded.length,
			initialVersionFound: loaded.some((version) => version.id === initialVersionId),
		})
		versions.value = loaded
		loadingVersions.value = false
	}

	function openUpdate(item: ContentItem) {
		const projectId = item.project?.id
		if (!ctx.installVersion || item.locked || !item.has_update || !projectId || !item.version?.id)
			return
		void open({ kind: 'item', item }, item.update_version_id ?? undefined, () =>
			fetchProjectVersions(projectId),
		)
	}

	function openSwitch(item: ContentItem) {
		const projectId = item.project?.id
		const versionId = item.version?.id
		if (!ctx.installVersion || item.locked || !projectId || !versionId) return
		if (!ctx.permissions(item).changeVersion) return
		void open({ kind: 'item', item }, versionId, () => fetchProjectVersions(projectId), true)
	}

	function openModpack() {
		const managed = ctx.managed.value
		const update = managed?.update
		if (!update || managed.disabledReason) return
		void open(
			{ kind: 'modpack', update },
			update.initialVersionId ?? update.currentVersionId ?? undefined,
			update.fetchVersions,
		)
	}

	function reset() {
		activeRequestId = 0
		target.value = null
		versions.value = []
		loadingVersions.value = false
		loadingChangelog.value = false
	}

	async function loadChangelog(version: Version, showLoading: boolean) {
		if (version.changelog != null || isExternal.value) return
		const requestId = activeRequestId
		if (showLoading) loadingChangelog.value = true
		try {
			const fullVersion = await fetchVersion(version.id)
			if (isActive(requestId)) versions.value = mergeVersion(versions.value, fullVersion)
		} catch (error) {
			if (showLoading && isActive(requestId)) {
				notifyError(formatMessage(messages.failedToLoadChangelog), error)
			}
		} finally {
			if (showLoading && isActive(requestId)) loadingChangelog.value = false
		}
	}

	async function installModpackVersion() {
		const update = updatingModpack.value
		const version = pendingModpackVersion.value
		if (!update || !version || ctx.managed.value?.disabledReason) return
		pendingModpackVersion.value = null
		const requestId = activeRequestId
		updaterModal.value?.hide()
		await update.install(version).catch(() => undefined)
		if (isActive(requestId)) reset()
	}

	function cancelModpackUpdate() {
		pendingModpackVersion.value = null
	}

	async function selectVersion(version: Version, event?: MouseEvent) {
		const current = target.value
		if (!current || actionDisabled.value) return

		if (current.kind === 'modpack') {
			pendingModpackVersion.value = version
			const installed = versions.value.find(
				(candidate) => candidate.id === current.update.currentVersionId,
			)
			isDowngrade.value = installed
				? Date.parse(version.date_published) < Date.parse(installed.date_published)
				: false
			const needsConfirm =
				isDowngrade.value || versionChangesGameVersion(version, ctx.target.value.gameVersion)
			if (event?.shiftKey || ctx.skipOptionalConfirms?.value || !needsConfirm) {
				await installModpackVersion()
			} else {
				confirmModpackUpdateModal.value?.show()
			}
			return
		}

		const requestId = activeRequestId
		await ctx.installVersion?.(current.item, version).catch(() => undefined)
		if (isActive(requestId)) reset()
	}

	return {
		updaterModal,
		confirmModpackUpdateModal,
		updatingItem,
		updatingModpack,
		versions,
		loadingVersions,
		loadingChangelog,
		isDowngrade,
		currentVersionId,
		projectType,
		projectName,
		projectIconUrl,
		actionDisabled,
		actionDisabledTooltip,
		backupTip,
		openUpdate,
		openSwitch,
		openModpack,
		reset,
		loadChangelog,
		selectVersion,
		installModpackVersion,
		cancelModpackUpdate,
	}
}
