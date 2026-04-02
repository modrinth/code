<script setup lang="ts">
import { ServersManageOverviewPage } from '@modrinth/ui'
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
	isServerRunning: boolean
}

const props = defineProps<ServerProps>()
</script>

<template>
	<ServersManageOverviewPage v-bind="props" />
</template>
