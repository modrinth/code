<template>
	<CreationFlowModal
		ref="flow"
		type="server-onboarding"
		:search-projects="noProjects"
		:get-project-versions="noVersions"
		:on-back="() => flow?.hide()"
		@create="create"
	/>
</template>

<script setup lang="ts">
import {
	type CreationFlowContextValue,
	CreationFlowModal,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { nextTick, ref } from 'vue'

import { getInstanceIconUrl } from '@/helpers/instance'
import { exportInstancePack, pushPackToServer, serversQueryKey } from '@/helpers/linked-servers'
import type { GameInstance } from '@/helpers/types'

/**
 * Creates a server from this instance: the existing server setup, opened at its last step
 * (world settings), since the instance already decides the game version, loader, and content.
 */
const props = defineProps<{
	instance: GameInstance
}>()

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const flow = ref<InstanceType<typeof CreationFlowModal>>()

// The project search belongs to the first step, which this flow skips.
const noProjects = async () => ({ hits: [], total_hits: 0, offset: 0, limit: 0 })
const noVersions = async () => []

async function show() {
	await flow.value?.show()
	await nextTick()
	const ctx = flow.value?.ctx
	if (!ctx) return
	ctx.setupType.value = 'modpack'
	ctx.instanceSelection.value = {
		id: props.instance.id,
		name: props.instance.name,
		iconUrl: getInstanceIconUrl(props.instance.icon_path) ?? undefined,
	}
	ctx.modal.value?.setStage('final-config')
}

async function create(ctx: CreationFlowContextValue) {
	let serverId: string | undefined
	try {
		const pack = await exportInstancePack(props.instance.id)
		serverId = (await client.archon.servers_v1.createLocal(props.instance.name)).id
		await pushPackToServer(pack, { id: props.instance.id, name: props.instance.name }, serverId, {
			properties: ctx.buildProperties(),
		})
		flow.value?.hide()
	} catch (error) {
		// Do not leave an empty server behind when the instance never reached it.
		if (serverId) await client.archon.servers_v1.deleteLocal(serverId).catch(() => {})
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedTitle),
			text: error instanceof Error ? error.message : String(error),
		})
	} finally {
		ctx.loading.value = false
		await queryClient.invalidateQueries({ queryKey: serversQueryKey })
	}
}

const messages = defineMessages({
	failedTitle: {
		id: 'instance.linked-servers.create.failed-title',
		defaultMessage: 'Could not create the server',
	},
})

defineExpose({ show })
</script>
