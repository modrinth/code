<script setup lang="ts">
import { injectModrinthServerContext, ServersManageOverviewPage } from '@modrinth/ui'
import type { ServerState, Stats } from '@modrinth/utils'

type ServerProps = {
	isConnected: boolean
	isWsAuthIncorrect: boolean
	stats: Stats
	serverPowerState: ServerState
	powerStateDetails?: {
		oom_killed?: boolean
		exit_code?: number
	}
}

const props = defineProps<ServerProps>()
const { server } = injectModrinthServerContext()

useHead({
	title: computed(() => `Overview - ${server.value?.name ?? 'Server'} - Modrinth`),
})
</script>

<template>
	<ServersManageOverviewPage v-bind="props" />
</template>
