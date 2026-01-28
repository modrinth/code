<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import { ButtonStyled, commonMessages, defineMessages, useVIntl } from '@modrinth/ui'
import { getVersion } from '@tauri-apps/api/app'
import { onMounted, onUnmounted, ref } from 'vue'

const { formatMessage } = useVIntl()

const dismissed = ref(false)
const availableUpdate = ref<{ version: string } | null>(null)

let checkInterval: ReturnType<typeof setInterval> | null = null

async function checkForUpdate() {
	try {
		const [response, currentVersion] = await Promise.all([
			fetch('https://launcher-files.modrinth.com/updates.json'),
			getVersion(),
		])
		const updates = await response.json()
		const latestVersion = updates?.version

		if (latestVersion && latestVersion !== currentVersion) {
			if (latestVersion !== availableUpdate.value?.version) {
				availableUpdate.value = { version: latestVersion }
				dismissed.value = false
			}
		}
	} catch (e) {
		console.error('Failed to check for updates:', e)
	}
}

function dismiss() {
	dismissed.value = true
}

onMounted(() => {
	checkForUpdate()
	checkInterval = setInterval(checkForUpdate, 5 * 60 * 1000)
})

onUnmounted(() => {
	if (checkInterval) {
		clearInterval(checkInterval)
	}
})

const messages = defineMessages({
	title: {
		id: 'app.update-toast.title',
		defaultMessage: 'Update available',
	},
	body: {
		id: 'app.update-toast.body.linux',
		defaultMessage:
			'Modrinth App v{version} is available. Use your package manager to update for the latest features and fixes!',
	},
	download: {
		id: 'app.update-toast.download-page',
		defaultMessage: 'Download',
	},
})
</script>
<template>
	<div
		v-if="availableUpdate && !dismissed"
		class="grid grid-cols-[min-content] fixed card-shadow rounded-2xl top-[--top-bar-height] mt-6 right-6 p-4 z-10 bg-bg-raised border-divider border-solid border-[2px]"
	>
		<div class="flex min-w-[25rem] gap-4">
			<h2 class="whitespace-nowrap text-base text-contrast font-semibold m-0 grow">
				{{ formatMessage(messages.title) }}
			</h2>
			<ButtonStyled size="small" circular>
				<button v-tooltip="formatMessage(commonMessages.closeButton)" @click="dismiss">
					<XIcon />
				</button>
			</ButtonStyled>
		</div>
		<p class="text-sm mt-2 mb-0">
			{{ formatMessage(messages.body, { version: availableUpdate.version }) }}
		</p>
	</div>
</template>
