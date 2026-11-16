<template>
	<CreationFlowModal
		ref="modalRef"
		type="server-onboarding"
		:available-loaders="['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']"
		show-snapshot-toggle
		:get-loader-manifest="getLoaderManifest"
		:get-curse-forge-target="getCurseForgeTarget"
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
import CreationFlowModal from '#ui/components/flows/creation-flow-modal/index.vue'
import { installServerCreationConfig } from '#ui/components/flows/creation-flow-modal/server-install'
import { useVIntl } from '#ui/composables/i18n'
import { hasServerPermission } from '#ui/composables/server-permissions'
import { serverKeys } from '#ui/composables/server-queries'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

import { injectServerOnboardingFlow, type ServerOnboardingRequest } from './context'
import { useDismissServerIntro } from './use-dismiss-server-intro'

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
	queryKey: computed(() => serverKeys.detail(serverId.value)),
	queryFn: ({ queryKey }) => client.archon.servers_v0.get(queryKey[2]),
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

async function getCurseForgeTarget() {
	const request = flow.request.value
	if (!request) throw new Error('No server is being set up')
	const auth = await client.archon.servers_v0.getFilesystemAuth(request.serverId)
	return { worldId: request.worldId, auth }
}

function onBack() {
	if (!flow.request.value?.backToBrowse) return false
	modalRef.value?.hide()
}

async function onAfterHide() {
	if (opening) return
	const request = flow.request.value
	request?.onHide?.()
	await introCompletion?.catch(() => {})
	if (flow.request.value === request) flow.clear()
}

const install = useMutation({
	mutationFn: ({
		config,
		request,
	}: {
		config: CreationFlowContextValue
		request: ServerOnboardingRequest
	}) => installServerCreationConfig(client, config, request.serverId, request.worldId),
	onSuccess: (_result, { request }) => {
		queryClient.setQueryData<Archon.Servers.v0.Server>(
			serverKeys.detail(request.serverId),
			(server) => (server ? { ...server, status: 'installing' } : server),
		)
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
