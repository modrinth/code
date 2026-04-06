<template>
	<div class="contents">
		<NewModal
			ref="detailsModal"
			:header="`All of ${server.name || 'Server'} info`"
			@close="detailsModal?.hide()"
		>
			<ServerInfoLabels
				:server-data="server"
				:show-game-label="true"
				:show-loader-label="true"
				:uptime-seconds="props.uptimeSeconds"
				:column="true"
				class="mb-6 flex flex-col gap-2"
			/>
			<div v-if="props.showDebugInfo" class="markdown-body">
				<pre>{{ server }}</pre>
			</div>
			<ButtonStyled type="standard" color="brand" @click="detailsModal?.hide()">
				<button class="w-full">Close</button>
			</ButtonStyled>
		</NewModal>

		<ButtonStyled circular type="transparent" size="large">
			<TeleportOverflowMenu :options="menuOptions">
				<MoreVerticalIcon aria-hidden="true" />
				<template #kill>
					<SlashIcon class="h-5 w-5" />
					<span>Kill server</span>
				</template>
				<template #allServers>
					<ServerIcon class="h-5 w-5" />
					<span>All servers</span>
				</template>
				<template #details>
					<InfoIcon class="h-5 w-5" />
					<span>Details</span>
				</template>
				<template #copy-id>
					<ClipboardCopyIcon class="h-5 w-5" aria-hidden="true" />
					<span>Copy ID</span>
				</template>
			</TeleportOverflowMenu>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import {
	ClipboardCopyIcon,
	InfoIcon,
	MoreVerticalIcon,
	ServerIcon,
	SlashIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import { ButtonStyled, NewModal, ServerInfoLabels } from '#ui/components'
import TeleportOverflowMenu from '#ui/components/base/TeleportOverflowMenu.vue'
import { injectModrinthServerContext } from '#ui/providers'

import { useServerPowerAction } from './use-server-power-action'

const props = withDefaults(
	defineProps<{
		disabled?: boolean
		showCopyIdAction?: boolean
		showDebugInfo?: boolean
		uptimeSeconds?: number
	}>(),
	{
		disabled: false,
		showCopyIdAction: false,
		showDebugInfo: false,
		uptimeSeconds: 0,
	},
)

const router = useRouter()
const { serverId, server } = injectModrinthServerContext()

const detailsModal = ref<InstanceType<typeof NewModal> | null>(null)

const { isInstalling, initiateAction } = useServerPowerAction({
	disabled: computed(() => props.disabled),
})

const menuOptions = computed(() => [
	...(isInstalling.value
		? []
		: [
				{
					id: 'kill',
					label: 'Kill server',
					icon: SlashIcon,
					action: () => initiateAction('Kill'),
				},
			]),
	{
		id: 'allServers',
		label: 'All servers',
		icon: ServerIcon,
		action: () => router.push('/hosting/manage'),
	},
	{
		id: 'details',
		label: 'Details',
		icon: InfoIcon,
		action: () => detailsModal.value?.show(),
	},
	{
		id: 'copy-id',
		label: 'Copy ID',
		icon: ClipboardCopyIcon,
		action: () => copyId(),
		shown: props.showCopyIdAction,
	},
])

async function copyId() {
	await navigator.clipboard.writeText(serverId)
}
</script>
