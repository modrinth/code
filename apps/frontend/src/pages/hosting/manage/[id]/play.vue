<script setup lang="ts">
import {
	injectModrinthClient,
	injectModrinthServerContext,
	type ServerPlayTarget,
	ServersManagePlayPage,
} from '@modrinth/ui'

const client = injectModrinthClient()
const { server } = injectModrinthServerContext()
const config = useRuntimeConfig()
useHead({ title: computed(() => `Play - ${server.value?.name ?? 'Server'} - Modrinth`) })

function playServer({ serverId, worldId }: ServerPlayTarget) {
	window.location.assign(
		`modrinth://hosting/${encodeURIComponent(serverId)}/${encodeURIComponent(worldId)}`,
	)
}

async function downloadMrpack(instanceId: string, version: number, filename: string) {
	const blob = await client.sharedinstances.instances_v1.downloadMrpack(instanceId, version)
	const url = URL.createObjectURL(blob)
	const anchor = document.createElement('a')
	anchor.href = url
	anchor.download = filename
	document.body.appendChild(anchor)
	anchor.click()
	anchor.remove()
	setTimeout(() => URL.revokeObjectURL(url), 60_000)
}
</script>

<template>
	<ServersManagePlayPage
		:on-play-server="playServer"
		:on-download-mrpack="downloadMrpack"
		:site-url="config.public.siteUrl"
	/>
</template>
