import type { Archon, Labrinth } from '@modrinth/api-client'
import type {
	BrowseInstallContentType,
	BrowseInstallPlan,
	BrowseSearchState,
	CreationFlowContextValue,
	FilterValue,
	PendingServerContentInstall,
	PendingServerContentInstallType,
} from '@modrinth/ui'
import {
	addPendingServerContentInstalls,
	commonMessages,
	defineMessages,
	flushStoredServerAddonInstallQueue,
	getStoredServerAddonInstallQueue,
	getTargetInstallPreferences,
	injectModrinthClient,
	injectNotificationManager,
	readPendingServerContentInstalls,
	readStoredServerInstallQueue,
	removePendingServerContentInstall,
	requestInstall,
	useVIntl,
	writePendingServerContentInstallBaseline,
	writeStoredServerInstallQueue,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import type { ComputedRef, Ref } from 'vue'
import { computed, nextTick, ref, watch } from 'vue'

import { navigateTo, useRoute } from '#app'
import { queryAsString } from '~/utils/router'

type PendingServerContentInstallInput = Omit<PendingServerContentInstall, 'createdAt'>
type ServerInstallBrowseSearchState = Pick<
	BrowseSearchState,
	'currentFilters' | 'overriddenProvidedFilterTypes'
>
type ServerInstallProjectType = {
	id?: string
}
type ServerInstallDebug = (...args: unknown[]) => void

export interface ServerInstallModalHandle {
	show: () => void | Promise<void>
	hide: () => void
	ctx?: CreationFlowContextValue | null
}

export interface ServerInstallSearchResult extends Labrinth.Search.v2.ResultSearchProject {
	installed?: boolean
}

export interface UseServerInstallContentOptions {
	projectType: ComputedRef<ServerInstallProjectType | undefined>
	onboardingModalRef: Ref<ServerInstallModalHandle | null>
	debug?: ServerInstallDebug
}

const messages = defineMessages({
	unsupportedContentType: {
		id: 'discover.install.error.unsupported-content-type',
		defaultMessage: 'This content type cannot be installed to a server from browse.',
	},
	noServerWorld: {
		id: 'discover.install.error.no-server-world',
		defaultMessage: 'No server world is available for install.',
	},
	backToSetup: {
		id: 'discover.install.back-to-setup',
		defaultMessage: 'Back to setup',
	},
	cancelReset: {
		id: 'discover.install.cancel-reset',
		defaultMessage: 'Cancel reset',
	},
	backToServer: {
		id: 'discover.install.back-to-server',
		defaultMessage: 'Back to server',
	},
	resetModpackHeading: {
		id: 'discover.install.heading.reset-modpack',
		defaultMessage: 'Selecting modpack to install after reset',
	},
})

function getQueuedInstallOwnerFallback(project: ServerInstallSearchResult) {
	if (project.organization) {
		const ownerId = project.organization_id ?? project.organization
		return {
			id: ownerId,
			name: project.organization,
			type: 'organization' as const,
			link: `/organization/${ownerId}`,
		}
	}

	if (!project.author) return null

	const ownerId = project.author_id ?? project.author
	return {
		id: ownerId,
		name: project.author,
		type: 'user' as const,
		link: `/user/${ownerId}`,
	}
}

function getQueuedAddonInstallPlans(
	plans: Map<string, BrowseInstallPlan<ServerInstallSearchResult>>,
) {
	return Array.from(plans.values()).filter((plan) => plan.contentType !== 'modpack')
}

export function useServerInstallContent({
	projectType,
	onboardingModalRef,
	debug = () => {},
}: UseServerInstallContentOptions) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const route = useRoute()
	const { handleError } = injectNotificationManager()
	let browseSearchState: ServerInstallBrowseSearchState | null = null

	const currentServerId = computed(() => queryAsString(route.query.sid) || null)
	const fromContext = computed(() => queryAsString(route.query.from) || null)
	const currentWorldId = computed(() => queryAsString(route.query.wid) || null)

	const {
		data: serverData,
		isLoading: serverDataLoading,
		error: serverDataError,
	} = useQuery({
		queryKey: computed(() => ['servers', 'detail', currentServerId.value] as const),
		queryFn: () => {
			debug('serverData queryFn firing for:', currentServerId.value)
			return client.archon.servers_v0.get(currentServerId.value!)
		},
		enabled: computed(() => {
			const enabled = !!currentServerId.value
			debug('serverData enabled:', enabled)
			return enabled
		}),
	})

	watch(serverData, (val) =>
		debug('serverData changed:', val?.server_id, val?.name, val?.loader, val?.mc_version),
	)
	watch(serverDataLoading, (val) => debug('serverData loading:', val))
	watch(serverDataError, (val) => {
		if (val) debug('serverData error:', val)
	})

	const serverIcon = computed(() => {
		if (!currentServerId.value || !import.meta.client) return null
		return localStorage.getItem(`server-icon-${currentServerId.value}`)
	})

	const serverHideInstalled = ref(false)
	const hideSelectedServerInstalls = ref(false)
	const installingProjectIds = ref<Set<string>>(new Set())
	const optimisticallyInstalledProjectIds = ref<Set<string>>(new Set())
	const hiddenInstalledProjectIds = ref<Set<string>>(new Set())
	const hiddenInstalledProjectIdsInitialized = ref(false)
	const queuedServerInstalls = ref<Map<string, BrowseInstallPlan<ServerInstallSearchResult>>>(
		readStoredServerInstallQueue(currentServerId.value, currentWorldId.value),
	)
	const queuedServerInstallProjectIds = computed(() => new Set(queuedServerInstalls.value.keys()))
	const queuedServerInstallCount = computed(() => queuedServerInstalls.value.size)
	const selectedServerInstallProjects = computed(() =>
		Array.from(queuedServerInstalls.value.values()).map((plan) => ({
			id: plan.projectId,
			name: plan.project.title ?? formatMessage(commonMessages.projectLabel),
			iconUrl: plan.project.icon_url ?? null,
		})),
	)
	const isInstallingQueuedServerInstalls = ref(false)
	const queuedInstallProgress = ref({ completed: 0, total: 0 })
	const serverInstallQueue = {
		get: () => queuedServerInstalls.value,
		set: (plans: Map<string, BrowseInstallPlan<ServerInstallSearchResult>>) => {
			queuedServerInstalls.value = plans
			writeStoredServerInstallQueue(currentServerId.value, currentWorldId.value, plans)
		},
	}

	const contentQueryKey = computed(() => ['content', 'list', currentServerId.value ?? ''] as const)
	const { data: serverContentData, error: serverContentError } = useQuery({
		queryKey: contentQueryKey,
		queryFn: () =>
			client.archon.content_v1.getAddons(currentServerId.value!, currentWorldId.value!),
		enabled: computed(() => !!currentServerId.value && !!currentWorldId.value),
	})

	function setBrowseSearchState(state: ServerInstallBrowseSearchState) {
		browseSearchState = state
	}

	function setStoredServerInstallPlans(
		serverId: string,
		worldId: string,
		plans: Map<string, BrowseInstallPlan<ServerInstallSearchResult>>,
	) {
		if (serverId === currentServerId.value && worldId === currentWorldId.value) {
			queuedServerInstalls.value = plans
		}
		writeStoredServerInstallQueue(serverId, worldId, plans)
	}

	async function getQueuedInstallOwner(project: ServerInstallSearchResult) {
		const fallback = getQueuedInstallOwnerFallback(project)

		try {
			if (project.organization) {
				const organization = await client.labrinth.projects_v3.getOrganization(project.project_id)
				if (organization) {
					return {
						id: organization.id,
						name: organization.name,
						type: 'organization' as const,
						avatar_url: organization.icon_url ?? undefined,
						link: `/organization/${organization.slug}`,
					}
				}
			}

			const members = await client.labrinth.projects_v3.getMembers(project.project_id)
			const owner =
				members.find((member) => member.user.id === project.author_id)?.user ??
				members.find((member) => member.is_owner || member.role === 'Owner')?.user ??
				members[0]?.user

			if (owner) {
				return {
					id: owner.id,
					name: owner.username,
					type: 'user' as const,
					avatar_url: owner.avatar_url,
					link: `/user/${owner.username}`,
				}
			}
		} catch {
			return fallback
		}

		return fallback
	}

	function getQueuedInstallPlaceholder(
		plan: BrowseInstallPlan<ServerInstallSearchResult>,
		owner: PendingServerContentInstallInput['owner'],
	): PendingServerContentInstallInput {
		return {
			projectId: plan.projectId,
			versionId: plan.versionId,
			contentType: plan.contentType as PendingServerContentInstallType,
			title: plan.project.title ?? formatMessage(commonMessages.projectLabel),
			versionName: plan.versionName ?? null,
			versionNumber: plan.versionNumber ?? null,
			fileName: plan.fileName ?? null,
			owner,
			slug: plan.project.slug ?? plan.projectId,
			iconUrl: plan.project.icon_url ?? null,
		}
	}

	function getQueuedInstallPlaceholderFallbacks(
		plans: Map<string, BrowseInstallPlan<ServerInstallSearchResult>>,
	) {
		return getQueuedAddonInstallPlans(plans).map((plan) =>
			getQueuedInstallPlaceholder(plan, getQueuedInstallOwnerFallback(plan.project)),
		)
	}

	async function getQueuedInstallPlaceholders(
		plans: Map<string, BrowseInstallPlan<ServerInstallSearchResult>>,
	) {
		return Promise.all(
			getQueuedAddonInstallPlans(plans).map(async (plan) =>
				getQueuedInstallPlaceholder(plan, await getQueuedInstallOwner(plan.project)),
			),
		)
	}

	function setProjectInstalling(projectId: string, installing: boolean) {
		const next = new Set(installingProjectIds.value)
		if (installing) {
			next.add(projectId)
		} else {
			next.delete(projectId)
		}
		installingProjectIds.value = next
	}

	function markProjectInstalled(projectId: string) {
		optimisticallyInstalledProjectIds.value = new Set([
			...optimisticallyInstalledProjectIds.value,
			projectId,
		])
	}

	function getServerInstalledProjectIds(data = serverContentData.value) {
		return new Set(
			(data?.addons ?? [])
				.map((addon) => addon.project_id)
				.filter((projectId): projectId is string => !!projectId),
		)
	}

	function getServerInstalledContentKeys(data = serverContentData.value) {
		return new Set((data?.addons ?? []).map((addon) => addon.project_id ?? addon.filename))
	}

	function syncHiddenInstalledProjectIds() {
		hiddenInstalledProjectIds.value = new Set([
			...getServerInstalledProjectIds(),
			...optimisticallyInstalledProjectIds.value,
		])
		hiddenInstalledProjectIdsInitialized.value = true
	}

	const serverFilters = computed<FilterValue[]>(() => {
		debug(
			'serverFilters recomputing, serverData:',
			!!serverData.value,
			'projectType:',
			projectType.value?.id,
		)
		const filters: FilterValue[] = []
		if (serverData.value && projectType.value?.id !== 'modpack') {
			const gameVersion = serverData.value.mc_version
			if (gameVersion) {
				filters.push({ type: 'game_version', option: gameVersion })
			}

			const platform = serverData.value.loader?.toLowerCase()

			const modLoaders = ['fabric', 'forge', 'quilt', 'neoforge']
			if (platform && modLoaders.includes(platform)) {
				filters.push({ type: 'mod_loader', option: platform })
			}

			const pluginLoaders = ['paper', 'purpur']
			if (platform && pluginLoaders.includes(platform)) {
				filters.push({ type: 'plugin_loader', option: platform })
			}

			if (projectType.value?.id === 'mod') {
				filters.push({ type: 'environment', option: 'server' })
			}

			if (serverHideInstalled.value && hiddenInstalledProjectIds.value.size > 0) {
				for (const x of hiddenInstalledProjectIds.value) {
					filters.push({
						type: 'project_id',
						option: `project_id:${x}`,
						negative: true,
					})
				}
			}

			if (hideSelectedServerInstalls.value && queuedServerInstallProjectIds.value.size > 0) {
				for (const id of queuedServerInstallProjectIds.value) {
					filters.push({
						type: 'project_id',
						option: `project_id:${id}`,
						negative: true,
					})
				}
			}
		}

		if (currentServerId.value && projectType.value?.id === 'modpack') {
			filters.push(
				{ type: 'environment', option: 'client' },
				{ type: 'environment', option: 'server' },
			)
		}
		debug('serverFilters result:', filters)
		return filters
	})

	function getCurrentServerInstallType(): BrowseInstallContentType {
		const type = projectType.value?.id
		if (type === 'modpack' || type === 'mod' || type === 'plugin' || type === 'datapack') {
			return type
		}
		throw new Error(formatMessage(messages.unsupportedContentType))
	}

	function getServerInstallTargetPreferences(contentType: BrowseInstallContentType) {
		return getTargetInstallPreferences(
			{
				gameVersion: serverData.value?.mc_version,
				loader: serverData.value?.loader,
			},
			contentType,
		)
	}

	function getInstallProjectVersions(projectId: string) {
		return client.labrinth.versions_v2.getProjectVersions(projectId, {
			include_changelog: false,
		})
	}

	function clearQueuedServerInstalls() {
		serverInstallQueue.set(new Map())
	}

	function removeQueuedServerInstall(projectId: string) {
		const nextPlans = new Map(queuedServerInstalls.value)
		nextPlans.delete(projectId)
		serverInstallQueue.set(nextPlans)
	}

	async function flushQueuedServerInstalls(
		serverId: string | null = currentServerId.value,
		worldId: string | null = currentWorldId.value,
	) {
		if (isInstallingQueuedServerInstalls.value) return false

		if (!serverId || !worldId) {
			handleError(new Error(formatMessage(messages.noServerWorld)))
			return false
		}

		const queuedPlans = getStoredServerAddonInstallQueue<ServerInstallSearchResult>(
			serverId,
			worldId,
		)
		if (queuedPlans.size === 0) return true

		isInstallingQueuedServerInstalls.value = true
		queuedInstallProgress.value = {
			completed: 0,
			total: queuedPlans.size,
		}

		try {
			const result = await flushStoredServerAddonInstallQueue({
				serverId,
				worldId,
				install: (plans) =>
					client.archon.content_v1.addAddons(
						serverId,
						worldId,
						plans.map((plan) => ({
							project_id: plan.projectId,
							version_id: plan.versionId,
						})),
					),
				onQueueChange: (plans) => setStoredServerInstallPlans(serverId, worldId, plans),
			})

			if (!result.ok) {
				for (const plan of result.attemptedPlans) {
					removePendingServerContentInstall(serverId, worldId, plan.projectId)
				}
				handleError(result.error as Error)
				return false
			}

			for (const plan of result.flushedPlans) {
				markProjectInstalled(plan.projectId)
			}
			queuedInstallProgress.value = {
				completed: result.flushedPlans.length,
				total: result.flushedPlans.length,
			}
			if (result.flushedPlans.length > 0) {
				await Promise.all([
					queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', serverId] }),
					queryClient.invalidateQueries({ queryKey: ['content', 'list'] }),
				])
			}

			return true
		} finally {
			isInstallingQueuedServerInstalls.value = false
			queuedInstallProgress.value = { completed: 0, total: 0 }
		}
	}

	async function discardQueuedServerInstallsAndBack() {
		clearQueuedServerInstalls()
		await navigateTo(serverBackUrl.value)
	}

	async function installQueuedServerInstallsAndBack() {
		const sid = currentServerId.value
		const wid = currentWorldId.value
		const backUrl = serverBackUrl.value
		const plans = new Map(queuedServerInstalls.value)

		if (sid && wid) {
			writeStoredServerInstallQueue(sid, wid, plans)
			writePendingServerContentInstallBaseline(sid, wid, [
				...getServerInstalledContentKeys(),
				...optimisticallyInstalledProjectIds.value,
			])
			addPendingServerContentInstalls(sid, wid, getQueuedInstallPlaceholderFallbacks(plans))
			void getQueuedInstallPlaceholders(plans)
				.then((items) => {
					const pendingProjectIds = new Set(
						readPendingServerContentInstalls(sid, wid).map((item) => item.projectId),
					)
					addPendingServerContentInstalls(
						sid,
						wid,
						items.filter((item) => pendingProjectIds.has(item.projectId)),
					)
				})
				.catch((err) => handleError(err as Error))
		}
		await navigateTo(backUrl)
		void flushQueuedServerInstalls(sid, wid)

		return true
	}

	async function serverInstall(project: ServerInstallSearchResult) {
		if (!serverData.value || !currentServerId.value || !currentWorldId.value) {
			handleError(new Error('No server to install to.'))
			return
		}

		if (!browseSearchState) {
			handleError(new Error('Search state is not ready.'))
			return
		}

		const contentType = getCurrentServerInstallType()
		const isModpack = contentType === 'modpack'

		try {
			if (!isModpack && queuedServerInstallProjectIds.value.has(project.project_id)) {
				removeQueuedServerInstall(project.project_id)
				return
			}

			if (isModpack || !queuedServerInstallProjectIds.value.has(project.project_id)) {
				setProjectInstalling(project.project_id, true)
			}

			await requestInstall({
				project,
				contentType,
				mode: isModpack ? 'immediate' : 'queue',
				selectedFilters: isModpack ? [] : browseSearchState.currentFilters.value,
				providedFilters: isModpack ? [] : serverFilters.value,
				overriddenProvidedFilterTypes: isModpack
					? []
					: browseSearchState.overriddenProvidedFilterTypes.value,
				targetPreferences: getServerInstallTargetPreferences(contentType),
				getProjectVersions: getInstallProjectVersions,
				queue: serverInstallQueue,
				install: async (plan) => {
					const modalInstance = onboardingModalRef.value
					if (!modalInstance) {
						setProjectInstalling(plan.projectId, false)
						return
					}

					onboardingInstallingProject.value = plan.project
					modalInstance.show()
					await nextTick()
					const ctx = modalInstance.ctx
					if (!ctx) return

					ctx.setupType.value = 'modpack'
					ctx.modpackSelection.value = {
						projectId: plan.projectId,
						versionId: plan.versionId,
						name: plan.project.title,
						iconUrl: plan.project.icon_url ?? undefined,
					}
					ctx.modal.value?.setStage('final-config')
				},
			})
		} catch (e) {
			console.error(e)
			if (isModpack) {
				setProjectInstalling(project.project_id, false)
			}
			handleError(e instanceof Error ? e : new Error(`Error installing content ${e}`))
		} finally {
			if (!isModpack) {
				setProjectInstalling(project.project_id, false)
			}
		}
	}

	const onboardingInstallingProject = ref<ServerInstallSearchResult | null>(null)

	function onOnboardingHide() {
		if (onboardingInstallingProject.value) {
			setProjectInstalling(onboardingInstallingProject.value.project_id, false)
			onboardingInstallingProject.value = null
		}
	}

	function onOnboardingBack() {
		onboardingModalRef.value?.hide()
	}

	async function onModpackFlowCreate(config: CreationFlowContextValue) {
		if (!currentServerId.value || !currentWorldId.value || !config.modpackSelection.value) return

		try {
			await client.archon.content_v1.installContent(currentServerId.value, currentWorldId.value, {
				content_variant: 'modpack',
				spec: {
					platform: 'modrinth',
					project_id: config.modpackSelection.value.projectId,
					version_id: config.modpackSelection.value.versionId,
				},
				soft_override: false,
				properties: config.buildProperties(),
			} satisfies Archon.Content.v1.InstallWorldContent)

			if (fromContext.value === 'onboarding') {
				await client.archon.servers_v1.endIntro(currentServerId.value)
				queryClient.invalidateQueries({ queryKey: ['servers', 'detail', currentServerId.value] })
				navigateTo(`/hosting/manage/${currentServerId.value}/content`)
			} else {
				navigateTo(`/hosting/manage/${currentServerId.value}?openSettings=installation`)
			}
		} catch (e) {
			handleError(new Error(`Error installing modpack: ${e}`))
			config.loading.value = false
		}
	}

	const serverBackUrl = computed(() => {
		if (!serverData.value) return ''
		const id = serverData.value.server_id
		if (fromContext.value === 'onboarding') return `/hosting/manage/${id}?resumeModal=setup-type`
		if (fromContext.value === 'reset-server')
			return `/hosting/manage/${id}?openSettings=installation`
		return `/hosting/manage/${id}/content`
	})

	const serverBackLabel = computed(() => {
		if (fromContext.value === 'onboarding') return formatMessage(messages.backToSetup)
		if (fromContext.value === 'reset-server') return formatMessage(messages.cancelReset)
		return formatMessage(messages.backToServer)
	})

	const serverBrowseHeading = computed(() =>
		fromContext.value === 'reset-server'
			? formatMessage(messages.resetModpackHeading)
			: formatMessage(commonMessages.installingContentLabel),
	)

	const installContext = computed(() => {
		if (!serverData.value) return null
		return {
			name: serverData.value.name,
			loader: serverData.value.loader ?? '',
			gameVersion: serverData.value.mc_version ?? '',
			serverId: currentServerId.value,
			upstream: serverData.value.upstream,
			iconSrc: serverIcon.value,
			isMedal: serverData.value.is_medal,
			backUrl: serverBackUrl.value,
			backLabel: serverBackLabel.value,
			heading: serverBrowseHeading.value,
			queuedCount: queuedServerInstallCount.value,
			selectedProjects: selectedServerInstallProjects.value,
			isInstallingSelected: isInstallingQueuedServerInstalls.value,
			installProgress: queuedInstallProgress.value,
			clearQueued: clearQueuedServerInstalls,
			clearSelected: clearQueuedServerInstalls,
			onBack: flushQueuedServerInstalls,
			discardSelectedAndBack: discardQueuedServerInstallsAndBack,
			installSelected: installQueuedServerInstallsAndBack,
		}
	})

	watch(serverContentError, (error) => {
		if (error) {
			console.error('Failed to load server content:', error)
			handleError(error)
		}
	})

	watch(
		serverContentData,
		(data) => {
			if (!data) return
			if (!hiddenInstalledProjectIdsInitialized.value) {
				syncHiddenInstalledProjectIds()
			}
		},
		{ immediate: true },
	)

	if (route.query.shi && projectType.value?.id !== 'modpack') {
		serverHideInstalled.value = route.query.shi === 'true'
	}

	watch(serverHideInstalled, (hideInstalled) => {
		if (hideInstalled) {
			syncHiddenInstalledProjectIds()
		}
	})

	watch([currentServerId, currentWorldId], ([serverId, worldId], [prevServerId, prevWorldId]) => {
		if (serverId !== prevServerId || worldId !== prevWorldId) {
			queuedServerInstalls.value = readStoredServerInstallQueue(serverId, worldId)
		}
	})

	watch(queuedServerInstallCount, (count) => {
		if (count === 0) {
			hideSelectedServerInstalls.value = false
		}
	})

	return {
		currentServerId,
		fromContext,
		currentWorldId,
		serverData,
		serverContentData,
		serverFilters,
		serverHideInstalled,
		hideSelectedServerInstalls,
		installingProjectIds,
		optimisticallyInstalledProjectIds,
		queuedServerInstallProjectIds,
		queuedServerInstallCount,
		isInstallingQueuedServerInstalls,
		installContext,
		setBrowseSearchState,
		syncHiddenInstalledProjectIds,
		serverInstall,
		onOnboardingHide,
		onOnboardingBack,
		onModpackFlowCreate,
	}
}
