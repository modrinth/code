<template>
	<CreationFlowModal
		ref="modalRef"
		type="server-onboarding"
		:available-loaders="['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']"
		show-snapshot-toggle
		:get-loader-manifest="getLoaderManifest"
		:finish-disabled="!canSetup"
		:finish-disabled-tooltip="
			!canSetup ? formatMessage(commonMessages.noPermissionAction) : undefined
		"
		:browse-modpacks="browseModpacks"
		:on-back="onBack"
		@create="onCreate"
		@after-hide="onAfterHide"
		@after-show="flow.markShown"
	/>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { until } from '@vueuse/core'
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { isNavigationFailure, NavigationFailureType, type RouteLocationRaw } from 'vue-router'

import type {
	CreationFlowContextValue,
	LoaderManifestResolver,
} from '#ui/components/flows/creation-flow-modal/creation-flow-context'
import { useVIntl } from '#ui/composables/i18n'
import { useDismissServerIntro } from '#ui/composables/server-onboarding'
import { hasServerPermission } from '#ui/composables/server-permissions'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import {
	injectServerOnboardingFlow,
	type ServerOnboardingRequest,
} from '#ui/providers/server-onboarding'
import { commonMessages } from '#ui/utils/common-messages'

import CreationFlowModal from '../flows/creation-flow-modal/index.vue'

const props = defineProps<{
	browsePath: string
	navigate: (to: RouteLocationRaw) => Promise<unknown>
	pageReady: boolean
	getLoaderManifest?: LoaderManifestResolver
}>()

const flow = injectServerOnboardingFlow()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const dismissServerIntro = useDismissServerIntro()
const modalRef = useTemplateRef<InstanceType<typeof CreationFlowModal>>('modalRef')
const serverId = computed(() => flow.request.value?.serverId ?? '')
const { data: server } = useQuery({
	queryKey: computed(() => ['servers', 'detail', serverId.value]),
	queryFn: () => client.archon.servers_v0.get(serverId.value),
	enabled: computed(() => !!serverId.value),
	staleTime: 30_000,
})
const canSetup = computed(() =>
	hasServerPermission(server.value?.current_user_permissions ?? 0, 'SETUP'),
)
let opening = false
let introCompletion: Promise<void> | null = null

async function navigateAndWaitForPage(to: RouteLocationRaw) {
	const result = await props.navigate(to)
	if (
		result === false ||
		(isNavigationFailure(result) && !isNavigationFailure(result, NavigationFailureType.duplicated))
	) {
		throw result || new Error('Navigation was cancelled')
	}
	await nextTick()
	await until(() => props.pageReady).toBe(true)
	await nextTick()
}

async function browseModpacks() {
	const request = flow.request.value
	if (!request) return
	await navigateAndWaitForPage({
		path: props.browsePath,
		query: { sid: request.serverId, wid: request.worldId, from: 'onboarding' },
	})
}

function onBack() {
	if (flow.request.value?.backToBrowse) {
		modalRef.value?.hide()
	} else {
		modalRef.value?.ctx.modal.value?.prevStage()
	}
}

async function onAfterHide() {
	if (opening) return
	const request = flow.request.value
	request?.onHide?.()
	await introCompletion?.catch(() => {})
	if (flow.request.value === request) flow.clear()
}

const install = useMutation({
	mutationFn: async ({
		config,
		request,
	}: {
		config: CreationFlowContextValue
		request: ServerOnboardingRequest
	}) => {
		const { serverId, worldId } = request
		if (config.projectInstall.value) {
			await config.installServerContent(serverId, worldId)
		} else if (config.setupType.value === 'modpack' && config.modpackFile.value) {
			config.uploadProgress.value = 0
			const upload = client.kyros.content_v1.uploadModpackFile(
				worldId,
				config.modpackFile.value,
				config.buildProperties(),
				{
					softOverride: true,
					onProgress: ({ loaded, total }) => {
						config.uploadProgress.value = total > 0 ? Math.round((loaded / total) * 100) : 0
					},
				},
			)
			await upload.promise
		} else {
			const selection = config.modpackSelection.value
			const loader = config.selectedLoader.value
			const content: Archon.Content.v1.InstallWorldContent =
				config.setupType.value === 'modpack' && selection
					? {
							content_variant: 'modpack',
							spec: {
								platform: 'modrinth',
								project_id: selection.projectId,
								version_id: selection.versionId,
							},
							soft_override: false,
							properties: config.buildProperties(),
						}
					: {
							content_variant: 'bare',
							loader: (loader === 'neoforge'
								? 'neo_forge'
								: (loader ?? 'vanilla')) as Archon.Content.v1.Modloader,
							version: config.selectedLoaderVersion.value ?? '',
							game_version: config.selectedGameVersion.value ?? undefined,
							soft_override: false,
							properties: config.buildProperties(),
						}
			await client.archon.content_v1.installContent(serverId, worldId, content)
		}
	},
	onSuccess: (_result, { request }) => {
		queryClient.setQueryData<Archon.Servers.v0.Server>(
			['servers', 'detail', request.serverId],
			(server) => (server ? { ...server, status: 'installing' } : server),
		)
	},
	onSettled: (_data, _error, { config }) => {
		config.uploadProgress.value = null
	},
})

async function onCreate(config: CreationFlowContextValue) {
	if (install.isPending.value) return
	const request = flow.request.value
	if (!request || !canSetup.value) {
		config.loading.value = false
		return
	}

	try {
		await install.mutateAsync({ config, request })
	} catch (error) {
		config.loading.value = false
		handleError(error as Error)
		return
	}

	flow.inviteActive.value = true
	const navigateToOverview = () => navigateAndWaitForPage(`/hosting/manage/${request.serverId}`)
	config.navigating.value = true
	config.showInvite(request.serverId, request.worldId, request.siteUrl, () => {
		introCompletion = dismissServerIntro.mutateAsync(request.serverId).then(() => {})
		return introCompletion
	})
	try {
		await navigateToOverview()
	} catch (error) {
		handleError(error as Error)
	} finally {
		config.navigating.value = false
	}
}

watch(
	() => flow.request.value,
	async (request) => {
		if (!request) return
		opening = true
		introCompletion = null
		try {
			await nextTick()
			if (!modalRef.value) throw new Error('Server setup modal is unavailable')
			await modalRef.value.show(async (ctx) => {
				const project = request.project
				if (!project) return
				if (project.contentType && project.contentType !== 'modpack') {
					await ctx.selectProject(project.projectId, project.contentType, project.versionId)
				} else {
					ctx.setupType.value = 'modpack'
					ctx.modpackSelection.value = project
					ctx.modal.value?.setStage('final-config')
				}
			})
		} catch (error) {
			flow.fail(error)
		} finally {
			opening = false
		}
	},
	{ immediate: true },
)
</script>
