<script setup lang="ts">
import { injectModrinthServerContext, type ServerPlayTarget, ServersManagePlayPage } from '@modrinth/ui'

const { server } = injectModrinthServerContext()
const config = useRuntimeConfig()
useHead({ title: computed(() => `Play - ${server.value?.name ?? 'Server'} - Modrinth`) })

function playServer({ serverId, worldId }: ServerPlayTarget) {
	window.location.assign(`modrinth://hosting/${encodeURIComponent(serverId)}/${encodeURIComponent(worldId)}`)
}

async function downloadMrpack(blob: Blob, filename: string) {
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
	<ServersManagePlayPage :on-play-server="playServer" :on-download-mrpack="downloadMrpack" :site-url="config.public.siteUrl" />
</template>
