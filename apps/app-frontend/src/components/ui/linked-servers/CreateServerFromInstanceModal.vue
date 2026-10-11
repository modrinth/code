<template>
	<CreationFlowModal
		ref="flow"
		type="server-onboarding"
		:search-projects="noProjects"
		:get-project-versions="noVersions"
		:on-back="() => flow?.hide()"
		@create="create"
	/>
	<ChooseCoreModal ref="chooseCore" />
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
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

import ChooseCoreModal from '@/components/ui/cores/ChooseCoreModal.vue'
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
const chooseCore = ref<InstanceType<typeof ChooseCoreModal>>()
// The Core the server goes on, chosen before setup opens.
let core: Archon.Cores.v1.Core | null = null

// The project search belongs to the first step, which this flow skips.
const noProjects = async () => ({ hits: [], total_hits: 0, offset: 0, limit: 0 })
const noVersions = async () => []

async function show() {
	core = (await chooseCore.value?.pick()) ?? null
	if (!core) return
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

/**
 * The server exists as soon as the Core makes it, linked to the instance. If the upload then
 * fails the server stays, and the instance shows a banner to push again.
 */
async function create(ctx: CreationFlowContextValue) {
	const instance = { id: props.instance.id, name: props.instance.name }
	try {
		if (!core) throw new Error('No Core was chosen')
		const { id } = await client.archon.cores_v1.createServer(instance.name, core, instance)
		flow.value?.hide()
		await queryClient.invalidateQueries({ queryKey: serversQueryKey })
		try {
			const pack = await exportInstancePack(instance.id)
			await pushPackToServer(pack, instance, id, { properties: ctx.buildProperties() })
		} catch (error) {
			console.error('Failed to upload the instance to its new server', error)
		}
	} catch (error) {
		console.error('Failed to create server', error)
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedTitle),
			text: formatMessage(messages.failedText),
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
	failedText: {
		id: 'instance.linked-servers.create.failed-text',
		defaultMessage: 'Check that your Core is running, then try again.',
	},
})

defineExpose({ show })
</script>
