<template>
	<Teleport v-if="pipRoot" :to="pipRoot">
		<ModerationReviewPanels dock="pip" />
	</Teleport>
</template>

<script setup lang="ts">
import { injectNotificationManager } from '@modrinth/ui'

import { useModerationReviewLayout } from '~/services/moderation/review-layout'

import ModerationReviewPanels from './ModerationReviewPanels.vue'
import { closePipWindow, ensurePipWindow, pipSupported } from './pip-window'

const layout = useModerationReviewLayout()
const { addNotification } = injectNotificationManager()

const pipRoot = ref<HTMLElement | null>(null)

let opening = false

async function openPipWindow() {
	if (opening || pipRoot.value) return
	opening = true
	try {
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
			onClose: () => {
				pipRoot.value = null
				layout.reclaimPipTabs()
			},
		}).catch(() => null)

		if (!root) {
			layout.reclaimPipTabs()
			return
		}

		pipRoot.value = root
	} finally {
		opening = false
	}
}

/**
 * This host stays mounted for the whole review shell's lifetime (`ModerationReviewLayout.vue`
 * no longer gates it behind `v-if="pipOpen"`) — it reacts to `pipOpen` instead of being
 * created/destroyed by it. The real PiP window is a module-level singleton (see pip-window.ts,
 * deliberately not tied to any one component's lifecycle so it survives the `[type]/[project].vue`
 * remount on every queue navigation); a component whose *mount/unmount* opens/closes that window
 * is fragile — any spurious unmount (even one Vue coalesces away before it touches the DOM) would
 * request a *fresh* window outside of the user gesture `documentPictureInPicture.requestWindow()`
 * requires, which fails and silently dumps every PiP tab back into the main dock.
 */
watch(
	() => layout.pipOpen.value,
	(open) => {
		if (open) {
			void openPipWindow()
		} else if (pipRoot.value) {
			closePipWindow()
			pipRoot.value = null
		}
	},
	{ immediate: true },
)
</script>
