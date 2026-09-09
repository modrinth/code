<template>
	<Teleport v-if="pipRoot" :to="pipRoot">
		<ModerationReviewPanels dock="pip" />
	</Teleport>
</template>

<script setup lang="ts">
import { injectNotificationManager } from '@modrinth/ui'
import { onMounted, onUnmounted, ref } from 'vue'

import { useModerationReviewLayout } from '~/services/moderation/review-layout'

import ModerationReviewPanels from './ModerationReviewPanels.vue'
import { closePipWindow, ensurePipWindow, pipSupported } from './pip-window'

const layout = useModerationReviewLayout()
const { addNotification } = injectNotificationManager()

const pipRoot = ref<HTMLElement | null>(null)

onMounted(async () => {
	if (!pipSupported) {
		addNotification({
			title: 'Picture-in-Picture unavailable',
			text: 'The moderation PiP window requires a Chromium-based browser.',
			type: 'warning',
		})
		layout.reclaimPipTabs()
		return
	}

	const root = await ensurePipWindow({
		onClose: () => layout.reclaimPipTabs(),
	}).catch(() => null)

	if (!root) {
		layout.reclaimPipTabs()
		return
	}

	pipRoot.value = root
})

onUnmounted(() => {
	// Vue has already torn down the teleported panels by now. Keep the window alive across a
	// project-navigation remount (pipOpen stays true); only close it when the moderator
	// actually dismissed PiP.
	if (!layout.pipOpen.value) closePipWindow()
})
</script>
