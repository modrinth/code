<template>
	<div class="contents">
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
				<template #copy-id>
					<ClipboardCopyIcon class="h-5 w-5" aria-hidden="true" />
					<span>Copy ID</span>
				</template>
			</TeleportOverflowMenu>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { ClipboardCopyIcon, MoreVerticalIcon, ServerIcon, SlashIcon } from '@modrinth/assets'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import { ButtonStyled } from '#ui/components'
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
const { serverId } = injectModrinthServerContext()

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
