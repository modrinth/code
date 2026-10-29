<script setup lang="ts">
import videoSrc from '@modrinth/assets/illustrations/server-sharing/server-play-demo.webm'
import posterSrc from '@modrinth/assets/illustrations/server-sharing/server-play-demo.webp'
import { ServerSharingUpdateModal } from '@modrinth/ui'
import { useTemplateRef } from 'vue'

import { config } from '@/config'

import { useNewUpdateNotification } from './use-notification'

const modal = useTemplateRef<InstanceType<typeof ServerSharingUpdateModal>>('modal')
const { notifyForVersion, inviteTarget } = useNewUpdateNotification(show)

function show() {
	modal.value?.show(inviteTarget.value)
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide, notifyForVersion })
</script>

<template>
	<ServerSharingUpdateModal
		ref="modal"
		:site-url="config.siteUrl"
		:video-src="videoSrc"
		:poster-src="posterSrc"
	/>
</template>
