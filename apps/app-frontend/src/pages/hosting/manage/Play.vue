<script setup lang="ts">
import { injectServerPlay, ServersManagePlayPage } from '@modrinth/ui'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'

import { config } from '@/config'

const { play } = injectServerPlay()
async function downloadMrpack(blob: Blob, filename: string) {
	const path = await save({ defaultPath: filename, filters: [{ name: 'Modrinth modpack', extensions: ['mrpack'] }] })
	if (path) await writeFile(path, new Uint8Array(await blob.arrayBuffer()))
}
</script>

<template>
	<ServersManagePlayPage :on-play-server="play" :on-download-mrpack="downloadMrpack" :site-url="config.siteUrl" />
</template>
