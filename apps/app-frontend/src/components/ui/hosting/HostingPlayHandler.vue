<template>
	<ModrinthAccountRequiredModal ref="accountModal" :request-auth="requestAuth" />
	<SharedInstanceInstallModal ref="installModal" />
	<ContentDiffModal
		ref="updateModal"
		:header="formatMessage(messages.update)"
		:admonition-header="formatMessage(messages.update)"
		:description="formatMessage(messages.updateDescription)"
		:diffs="updateDiffs"
		:confirm-label="formatMessage(messages.update)"
		:confirm-icon="DownloadIcon"
		show-external-warnings
		@confirm="confirmUpdate"
	/>
</template>

<script setup lang="ts">
import { DownloadIcon } from '@modrinth/assets'
import { ContentDiffModal, type ContentDiffItem, getHostingServerAddress, defineMessages, injectAuth, injectModrinthClient, injectNotificationManager, type ServerPlayTarget, useVIntl } from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import ModrinthAccountRequiredModal from '@/components/ui/modal/ModrinthAccountRequiredModal.vue'
import SharedInstanceInstallModal from '@/components/ui/shared-instances/shared-instance-install-modal/index.vue'
import { toError } from '@/helpers/errors'
import { install_job_list, install_get_shared_instance_preview, install_get_shared_instance_update_preview, install_shared_instance, install_update_shared_instance, installJobInstanceId, wait_for_install_job } from '@/helpers/install'
import { get, list } from '@/helpers/instance'
import { get as getCredentials, type ModrinthAuthFlow } from '@/helpers/mr_auth'
import { ensureManagedServerWorldExists, start_join_server } from '@/helpers/worlds'
import { injectAppEvents } from '@/providers/app-events'

type LaunchTarget = ServerPlayTarget & { sharedInstanceId: string; name: string; address: string; userId: string; icon: string | null }
const auth = injectAuth()
const client = injectModrinthClient()
const appEvents = injectAppEvents()
const queryClient = useQueryClient()
const router = useRouter()
const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const accountModal = ref<InstanceType<typeof ModrinthAccountRequiredModal>>()
const installModal = ref<InstanceType<typeof SharedInstanceInstallModal>>()
const updateModal = ref<InstanceType<typeof ContentDiffModal>>()
const updateDiffs = ref<ContentDiffItem[]>([])
const pendingUpdate = ref<{ target: LaunchTarget; instanceId: string }>()

async function assertAccount(target: LaunchTarget) {
	if ((await getCredentials())?.user_id !== target.userId) throw new Error(formatMessage(messages.accountChanged))
}
async function findInstance(target: LaunchTarget) {
	const instance = (await list()).find((instance) => instance.shared_instance?.id === target.sharedInstanceId && instance.shared_instance.linked_user_id === target.userId)
	if (instance && instance.install_stage !== 'installed') {
		const job = (await install_job_list(false)).find((job) => installJobInstanceId(job) === instance.id)
		if (job) {
			await wait_for_install_job(appEvents, job.job_id)
			await assertAccount(target)
			return await get(instance.id) ?? undefined
		}
	}
	return instance
}
async function join(target: LaunchTarget, instanceId: string) {
	await assertAccount(target)
	const server = await client.archon.servers_v1.get(target.serverId)
	if (!server.worlds.some((world) => world.id === target.worldId && world.is_active && world.content?.shared_instance_id === target.sharedInstanceId)) {
		throw new Error(formatMessage(messages.worldChanged))
	}
	await assertAccount(target)
	const instance = await get(instanceId)
	if (!instance || instance.quarantined || instance.install_stage !== 'installed') throw new Error(formatMessage(messages.notReady))
	const legacy = await client.archon.servers_v0.get(target.serverId)
	const address = getHostingServerAddress(legacy.net, server.subdomain)
	if (!address) throw new Error(formatMessage(messages.noAddress))
	await assertAccount(target)
	await ensureManagedServerWorldExists(instanceId, target.name, address)
	await router.push(`/instance/${encodeURIComponent(instanceId)}`)
	await start_join_server(instanceId, address)
}
const launchMutation = useMutation({
	mutationFn: async ({ target, instanceId }: { target: LaunchTarget; instanceId?: string }) => {
		await assertAccount(target)
		const existing = await findInstance(target)
		if (instanceId && existing?.id !== instanceId) throw new Error(formatMessage(messages.notReady))
		if (existing) {
			if (existing.quarantined || existing.install_stage !== 'installed') throw new Error(formatMessage(messages.notReady))
			const update = await install_get_shared_instance_update_preview(existing.id)
			await assertAccount(target)
			if (update?.updateAvailable) {
				if (!instanceId) {
					showUpdate(target, existing.id, update)
					return
				}
				const job = await install_update_shared_instance(existing.id)
				await wait_for_install_job(appEvents, job.job_id)
			}
			await join(target, existing.id)
		} else {
			await assertAccount(target)
			const job = await install_shared_instance(target.sharedInstanceId, target.name, null, target.name, target.icon, target.icon)
			const installedId = installJobInstanceId(job)
			if (!installedId) throw new Error(formatMessage(messages.notReady))
			await queryClient.invalidateQueries({ queryKey: ['instances'] })
			await wait_for_install_job(appEvents, job.job_id)
			await join(target, installedId)
		}
	},
	onError: (error) => handleError(toError(error)),
	onSettled: () => queryClient.invalidateQueries({ queryKey: ['instances'] }),
})
function showUpdate(target: LaunchTarget, instanceId: string, preview: NonNullable<Awaited<ReturnType<typeof install_get_shared_instance_update_preview>>>) {
	pendingUpdate.value = { target, instanceId }
	updateDiffs.value = preview.diffs.map((diff) => ({
		type: diff.type, projectName: diff.projectName ?? undefined, fileName: diff.fileName ? encodeURIComponent(diff.fileName) : undefined,
		currentVersionName: diff.currentVersionName ?? undefined, newVersionName: diff.newVersionName ?? undefined,
		fileCount: diff.configFileCount ?? undefined, disabled: diff.disabled,
		external: diff.type === 'added' && !diff.projectId && !!diff.fileName,
	}))
	updateModal.value?.show()
}
function confirmUpdate() {
	if (!pendingUpdate.value || launchMutation.isPending.value) return
	launchMutation.mutate(pendingUpdate.value)
	pendingUpdate.value = undefined
}
const prepareMutation = useMutation({
	mutationFn: async ({ serverId, worldId }: ServerPlayTarget) => {
		if (auth.isReady && !auth.isReady.value) {
			await new Promise<void>((resolve) => {
				const stop = watch(auth.isReady!, (ready) => { if (ready) { stop(); resolve() } })
			})
		}
		if (!auth.session_token.value && !(await accountModal.value?.show())) return
		const credentials = await getCredentials()
		if (!credentials) return
		const [server, legacy] = await Promise.all([client.archon.servers_v1.get(serverId), client.archon.servers_v0.get(serverId)])
		const world = server.worlds.find((world) => world.id === worldId && world.is_active)
		const sharedInstanceId = world?.content?.shared_instance_id
		if (!sharedInstanceId) throw new Error(formatMessage(messages.worldChanged))
		const remote = await client.sharedinstances.instances_v1.get(sharedInstanceId)
		const address = getHostingServerAddress(legacy.net, server.subdomain)
		if (!address) throw new Error(formatMessage(messages.noAddress))
		const target: LaunchTarget = { serverId, worldId, sharedInstanceId, name: remote.name, address, userId: credentials.user_id, icon: remote.icon }
		await assertAccount(target)
		const existing = await findInstance(target)
		if (existing) {
			if (existing.quarantined || existing.install_stage !== 'installed') throw new Error(formatMessage(messages.notReady))
			const preview = await install_get_shared_instance_update_preview(existing.id)
			await assertAccount(target)
			if (preview?.updateAvailable) showUpdate(target, existing.id, preview)
			else await join(target, existing.id)
		} else {
			const preview = await install_get_shared_instance_preview(sharedInstanceId, target.name)
			await assertAccount(target)
			if (remote.icon) preview.iconUrl = remote.icon
			installModal.value?.show(preview, async () => {
				if (launchMutation.isPending.value) return
				await launchMutation.mutateAsync({ target }).catch(() => {})
			})
		}
	},
	onError: (error) => handleError(toError(error)),
})
async function play(target: ServerPlayTarget) {
	if (prepareMutation.isPending.value || launchMutation.isPending.value) return
	await prepareMutation.mutateAsync(target).catch(() => {})
}
async function requestAuth(flow: ModrinthAuthFlow) {
	await auth.requestSignIn('', flow, { showModal: false })
	return !!(await getCredentials())
}
watch(() => auth.user.value?.id, () => {
	installModal.value?.hide()
	updateModal.value?.hide()
	pendingUpdate.value = undefined
})
const messages = defineMessages({
	update: { id: 'hosting.play.update-to-play', defaultMessage: 'Update to play' },
	updateDescription: { id: 'hosting.play.update-description', defaultMessage: 'Update this instance to the server’s latest shared content before joining.' },
	accountChanged: { id: 'hosting.play.account-changed', defaultMessage: 'Your Modrinth account changed. Press Play server again to continue.' },
	worldChanged: { id: 'hosting.play.world-changed', defaultMessage: 'This world is no longer active or has not been shared. Open the server panel and press Play server again.' },
	notReady: { id: 'hosting.play.not-ready', defaultMessage: 'This instance is not available to launch. Check its installation status in your library.' },
	noAddress: { id: 'hosting.play.no-address', defaultMessage: 'This server does not have a connection address yet.' },
})
defineExpose({ play })
</script>
