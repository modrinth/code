import type { Archon, Labrinth } from '@modrinth/api-client'
import { useQueryClient } from '@tanstack/vue-query'
import type { Ref } from 'vue'
import { computed, nextTick, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import type { CreationFlowContextValue } from '#ui/components/flows/creation-flow-modal/creation-flow-context'
import { injectServerOnboardingFlow } from '#ui/components/servers/onboarding/context'
import { defineMessages, type MessageDescriptor, useVIntl } from '#ui/composables/i18n'
import {
	useServerContextRuntime,
	waitForServerContextRuntimeReady,
} from '#ui/composables/server-context-runtime'
import { useServerPanelSync } from '#ui/composables/server-panel-sync'
import { serverKeys } from '#ui/composables/server-queries'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'
import type { ServerPlatform } from '#ui/utils/loaders'
import type { FilterValue } from '#ui/utils/search'

import {
	type BrowseInstallContentType,
	type BrowseInstallPlan,
	type BrowseInstallPreferences,
	type BrowseInstallProject,
	type BrowseInstallQueue,
	flushStoredServerAddonInstallQueue,
	getServerAddonInstallPlanProjectIds,
	getStoredServerAddonInstallQueue,
	getTargetInstallPreferences,
	readStoredServerInstallQueue,
	requestInstall,
	resolveServerAddonInstallPlans,
	stripServerRuntimeInstallFilters,
	stripServerRuntimeInstallOverrides,
	writeStoredServerInstallQueue,
} from '../install'
import type { BrowseInstallState, BrowseSelectedProject, BrowseSelection } from '../types'

const messages = defineMessages({
	noServerWorld: {
		id: 'browse.install.error.no-server-world',
		defaultMessage: 'No server world is available for install.',
	},
	setupModalUnavailable: {
		id: 'browse.install.error.setup-modal-unavailable',
		defaultMessage: 'Server setup is unavailable.',
	},
	backToSetup: {
		id: 'browse.install.back-to-setup',
		defaultMessage: 'Back to setup',
	},
	cancelReset: {
		id: 'browse.install.cancel-reset',
		defaultMessage: 'Cancel reset',
	},
	backToServer: {
		id: 'browse.install.back-to-server',
		defaultMessage: 'Back to server',
	},
	resetModpackHeading: {
		id: 'browse.install.heading.reset-modpack',
		defaultMessage: 'Selecting content to install after reset',
	},
})

export type ServerInstallFrom = 'onboarding' | 'reset-server'

export function serverInstallHeadingMessage(resettingServer: boolean): MessageDescriptor {
	return resettingServer ? messages.resetModpackHeading : commonMessages.installingContentLabel
}

export interface ServerSetupModalHandle {
	show: () => void | Promise<void>
	hide: () => void
	ctx?: CreationFlowContextValue | null
}

export interface UseServerInstallOptions {
	serverId: Readonly<Ref<string | null>>
	worldId: Readonly<Ref<string | null>>
	from: Readonly<Ref<string | null>>
	platform: Readonly<Ref<ServerPlatform>>
	installedProjectIds: Readonly<Ref<Set<string>>>
	siteUrl: string
	setupModal?: Readonly<Ref<ServerSetupModalHandle | null>>
	onInstalled?: (serverId: string, projectIds: string[]) => void | Promise<void>
}

export interface ServerInstallRequest {
	contentType: BrowseInstallContentType
	getProjectVersions: (projectId: string) => Promise<Labrinth.Versions.v2.Version[]>
	selectedFilters?: readonly FilterValue[]
	providedFilters?: readonly FilterValue[]
	overriddenProvidedFilterTypes?: readonly string[]
}

export type ServerInstall = ReturnType<typeof useServerInstall>

export function useServerInstall(options: UseServerInstallOptions) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const onboardingFlow = injectServerOnboardingFlow()
	const { handleError } = injectNotificationManager()
	const queryClient = useQueryClient()
	const router = useRouter()

	useServerContextRuntime(options.serverId)
	useServerPanelSync({ serverId: options.serverId, worldId: options.worldId })

	const from = computed<ServerInstallFrom | null>(() =>
		options.from.value === 'onboarding' || options.from.value === 'reset-server'
			? options.from.value
			: null,
	)
	const isSetup = computed(() => !!options.serverId.value && !!from.value)

	const plans = ref(new Map<string, BrowseInstallPlan>())
	watch(
		[options.serverId, options.worldId],
		([serverId, worldId]) => {
			plans.value = readStoredServerInstallQueue(serverId, worldId)
		},
		{ immediate: true },
	)

	const rootProjectIds = computed(() => new Set(plans.value.keys()))
	const projectIds = computed(() => getServerAddonInstallPlanProjectIds(plans.value.values()))
	const installingProjectIds = ref(new Set<string>())
	const flushing = ref(false)
	const flowProjectId = ref<string | null>(null)

	function projectName(project: BrowseInstallProject) {
		return project.name || project.title || formatMessage(commonMessages.projectLabel)
	}

	const selectedProjects = computed<BrowseSelectedProject[]>(() =>
		Array.from(plans.value.values()).map((plan) => ({
			id: plan.projectId,
			name: projectName(plan.project),
			iconUrl: plan.project.icon_url ?? null,
		})),
	)

	const queue: BrowseInstallQueue = {
		get: () => plans.value,
		set: (next) => {
			plans.value = next
			writeStoredServerInstallQueue(options.serverId.value, options.worldId.value, next)
		},
	}

	function storePlans(serverId: string, worldId: string, next: Map<string, BrowseInstallPlan>) {
		if (serverId === options.serverId.value && worldId === options.worldId.value) {
			plans.value = next
		}
		writeStoredServerInstallQueue(serverId, worldId, next)
	}

	function remove(projectId: string) {
		const next = new Map(plans.value)
		next.delete(projectId)
		queue.set(next)
	}

	function clear() {
		queue.set(new Map())
	}

	function setInstalling(projectId: string, installing: boolean) {
		const next = new Set(installingProjectIds.value)
		if (installing) next.add(projectId)
		else next.delete(projectId)
		installingProjectIds.value = next
	}

	async function getUpdateChannel() {
		const serverId = options.serverId.value
		if (!serverId) return undefined

		const server = await queryClient.fetchQuery({
			queryKey: serverKeys.detailV1(serverId),
			queryFn: () => client.archon.servers_v1.get(serverId),
			staleTime: 30_000,
		})
		const world =
			server.worlds.find((candidate) => candidate.id === options.worldId.value) ??
			server.worlds.find((candidate) => candidate.is_active) ??
			server.worlds[0]
		return world?.update_channel
	}

	function targetPreferences(contentType: string): BrowseInstallPreferences {
		if (isSetup.value) return {}
		return getTargetInstallPreferences(options.platform.value, contentType)
	}

	function toResolvePreferences(
		preferences?: BrowseInstallPreferences,
	): Labrinth.Content.v3.ResolutionPreferences {
		return {
			game_versions: preferences?.gameVersions,
			loaders: preferences?.loaders,
		}
	}

	async function resolveAddonPlan(plan: BrowseInstallPlan, existingProjectIds: string[]) {
		const resolved = await client.labrinth.content_v3.resolve({
			project_id: plan.projectId,
			version_id: plan.versionId,
			content_type: plan.contentType as Labrinth.Content.v3.ContentType,
			selected: toResolvePreferences(plan.preferences),
			target: toResolvePreferences(targetPreferences(plan.contentType)),
			existing_project_ids: existingProjectIds,
			update_channel: await getUpdateChannel(),
		})

		return [resolved.primary, ...resolved.dependencies].map((item) => ({
			projectId: item.project_id,
			versionId: item.version_id,
		}))
	}

	async function resolveQueuedPlan(plan: BrowseInstallPlan) {
		const resolvedContent = await resolveAddonPlan(
			plan,
			Array.from(options.installedProjectIds.value),
		)
		const stored = plans.value.get(plan.projectId)
		if (!stored || stored.versionId !== plan.versionId) return

		const next = new Map(plans.value)
		next.set(plan.projectId, { ...stored, resolvedContent })
		queue.set(next)
	}

	async function flush(
		serverId: string | null = options.serverId.value,
		worldId: string | null = options.worldId.value,
	) {
		if (flushing.value) return false

		if (!serverId || !worldId) {
			handleError(new Error(formatMessage(messages.noServerWorld)))
			return false
		}

		if (getStoredServerAddonInstallQueue(serverId, worldId).size === 0) return true

		try {
			await waitForServerContextRuntimeReady(client, serverId)
		} catch (error) {
			handleError(error as Error)
			return false
		}

		flushing.value = true
		try {
			const result = await flushStoredServerAddonInstallQueue({
				serverId,
				worldId,
				install: async (queued) => {
					const addons = await resolveServerAddonInstallPlans({
						plans: queued,
						existingProjectIds: options.installedProjectIds.value,
						resolvePlan: resolveAddonPlan,
					})
					if (addons.length > 0) {
						await client.archon.content_v1.addAddons(serverId, worldId, addons)
					}
				},
				onQueueChange: (next) => storePlans(serverId, worldId, next),
			})

			if (!result.ok) {
				handleError(result.error as Error)
				return false
			}

			if (result.flushedPlans.length > 0) {
				await options.onInstalled?.(
					serverId,
					result.flushedPlans.map((plan) => plan.projectId),
				)
			}
			return true
		} finally {
			flushing.value = false
		}
	}

	const back = computed(() => {
		const serverId = options.serverId.value
		const to = !serverId
			? '/hosting/manage'
			: from.value === 'onboarding'
				? `/hosting/manage/${serverId}?resumeModal=setup-type`
				: from.value === 'reset-server'
					? `/hosting/manage/${serverId}?openSettings=installation`
					: `/hosting/manage/${serverId}/content`
		const label =
			from.value === 'onboarding'
				? messages.backToSetup
				: from.value === 'reset-server'
					? messages.cancelReset
					: messages.backToServer
		return { to, label: formatMessage(label) }
	})

	const heading = computed(() =>
		formatMessage(serverInstallHeadingMessage(from.value === 'reset-server')),
	)

	async function discardAndBack() {
		clear()
		await router.push(back.value.to)
	}

	async function installAndBack() {
		const serverId = options.serverId.value
		const worldId = options.worldId.value
		const to = back.value.to

		if (serverId && worldId) {
			writeStoredServerInstallQueue(serverId, worldId, new Map(plans.value))
		}
		if (!(await flush(serverId, worldId))) return false
		await router.push(to)
		return true
	}

	const selection: BrowseSelection = {
		projects: selectedProjects,
		installing: flushing,
		clear,
		installAndBack,
		discardAndBack,
	}

	function stateOf(
		projectId: string,
		{ installed, contentType }: { installed: boolean; contentType: string },
	): BrowseInstallState {
		if (installed) return 'installed'

		const queuedRoot = !isSetup.value && rootProjectIds.value.has(projectId)
		const queued = !isSetup.value && projectIds.value.has(projectId)
		const installing = installingProjectIds.value.has(projectId)

		if (installing || (flushing.value && queuedRoot)) {
			return installing && contentType !== 'modpack' && !flushing.value
				? 'validating'
				: 'installing'
		}
		if (queuedRoot) return 'selected'
		if (queued) return 'selected-dependency'
		return 'idle'
	}

	function onFlowHide() {
		if (flowProjectId.value) setInstalling(flowProjectId.value, false)
		flowProjectId.value = null
	}

	function onFlowBack() {
		options.setupModal?.value?.hide()
	}

	async function openInstallFlow(plan: BrowseInstallPlan) {
		const serverId = options.serverId.value
		const worldId = options.worldId.value
		if (!serverId || !worldId) throw new Error(formatMessage(messages.noServerWorld))

		const project = {
			projectId: plan.projectId,
			versionId: plan.versionId,
			contentType: plan.contentType,
			name: projectName(plan.project),
			iconUrl: plan.project.icon_url ?? undefined,
		}

		flowProjectId.value = plan.projectId
		setInstalling(plan.projectId, true)
		try {
			if (from.value === 'onboarding') {
				await onboardingFlow.open({
					serverId,
					worldId,
					siteUrl: options.siteUrl,
					project,
					backToBrowse: true,
					onHide: onFlowHide,
				})
				return
			}

			const modal = options.setupModal?.value
			if (!modal) throw new Error(formatMessage(messages.setupModalUnavailable))
			await modal.show()
			await nextTick()

			const flow = modal.ctx
			if (!flow) throw new Error(formatMessage(messages.setupModalUnavailable))

			if (plan.contentType !== 'modpack') {
				await flow.selectProject(plan.projectId, plan.contentType, plan.versionId)
				return
			}

			flow.setupType.value = 'modpack'
			flow.modpackSelection.value = {
				projectId: project.projectId,
				versionId: project.versionId,
				name: project.name,
				iconUrl: project.iconUrl,
			}
			flow.modal.value?.setStage('final-config')
		} catch (error) {
			onFlowHide()
			throw error
		}
	}

	async function onFlowCreate(config: CreationFlowContextValue) {
		const serverId = options.serverId.value
		const worldId = options.worldId.value
		if (!serverId || !worldId || (!config.modpackSelection.value && !config.projectInstall.value)) {
			config.loading.value = false
			return
		}

		try {
			if (config.projectInstall.value) {
				await config.installServerContent(serverId, worldId)
			} else {
				await client.archon.content_v1.installContent(serverId, worldId, {
					content_variant: 'modpack',
					spec: {
						platform: 'modrinth',
						project_id: config.modpackSelection.value!.projectId,
						version_id: config.modpackSelection.value!.versionId,
					},
					soft_override: false,
					properties: config.buildProperties(),
				} satisfies Archon.Content.v1.InstallWorldContent)
			}

			options.setupModal?.value?.hide()
			await router.push(`/hosting/manage/${serverId}?openSettings=installation`)
		} catch (error) {
			handleError(error as Error)
			config.loading.value = false
		}
	}

	async function install(project: BrowseInstallProject, request: ServerInstallRequest) {
		if (!options.serverId.value || !options.worldId.value) {
			handleError(new Error(formatMessage(messages.noServerWorld)))
			return
		}

		const projectId = project.project_id
		const isModpack = request.contentType === 'modpack'
		const queues = !isModpack && !isSetup.value

		if (queues && rootProjectIds.value.has(projectId)) {
			remove(projectId)
			return
		}
		if (queues && projectIds.value.has(projectId)) return

		setInstalling(projectId, true)
		try {
			const plan = await requestInstall({
				project,
				contentType: request.contentType,
				mode: queues ? 'queue' : 'immediate',
				selectedFilters: isModpack
					? []
					: stripServerRuntimeInstallFilters(request.selectedFilters ?? []),
				providedFilters: isModpack ? [] : (request.providedFilters ?? []),
				overriddenProvidedFilterTypes: isModpack
					? []
					: stripServerRuntimeInstallOverrides(request.overriddenProvidedFilterTypes ?? []),
				targetPreferences: targetPreferences(request.contentType),
				updateChannel: await getUpdateChannel(),
				getProjectVersions: request.getProjectVersions,
				queue,
				install: openInstallFlow,
			})
			if (queues) await resolveQueuedPlan(plan)
		} catch (error) {
			if (queues) remove(projectId)
			setInstalling(projectId, false)
			handleError(error as Error)
		} finally {
			if (queues) setInstalling(projectId, false)
		}
	}

	return {
		from,
		isSetup,
		back,
		heading,
		plans,
		rootProjectIds,
		projectIds,
		installingProjectIds,
		flushing,
		queue,
		selection,
		remove,
		clear,
		flush,
		stateOf,
		install,
		resolveQueuedPlan,
		getUpdateChannel,
		targetPreferences,
		openInstallFlow,
		onFlowHide,
		onFlowBack,
		onFlowCreate,
	}
}
