<script setup lang="ts">
import {
	injectFileDownload,
	injectModrinthServerContext,
	injectServerPlay,
	ServersManagePlayPage,
} from '@modrinth/ui'

import { config } from '@/config'

const { play } = injectServerPlay()
const fileDownload = injectFileDownload()
const { serverId, server } = injectModrinthServerContext()
async function downloadMrpack(instanceId: string, version: number, filename: string) {
	await fileDownload.download({
		type: 'mrpack',
		instanceId,
		version,
		filename,
		serverId,
		serverName: server.value.name,
	})
}
</script>

<template>
	<ServersManagePlayPage
		:on-play-server="play"
		:on-download-mrpack-external="downloadMrpack"
		:site-url="config.siteUrl"
	/>
</template>
