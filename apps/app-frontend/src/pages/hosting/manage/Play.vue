<script setup lang="ts">
import { injectServerPlay, ServersManagePlayPage } from '@modrinth/ui'

import { config } from '@/config'

const { play } = injectServerPlay()
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
	<ServersManagePlayPage
		:on-play-server="play"
		:on-download-mrpack="downloadMrpack"
		:site-url="config.siteUrl"
	/>
</template>
