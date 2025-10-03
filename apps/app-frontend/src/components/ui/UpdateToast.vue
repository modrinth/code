<script setup lang="ts">
import { DownloadIcon, ExternalIcon, RefreshCwIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, commonMessages, ProgressBar } from '@modrinth/ui'
import { formatBytes } from '@modrinth/utils'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { ref } from 'vue'

import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	(e: 'close' | 'restart' | 'download'): void
}>()

defineProps<{
	version: string
	size: number | null
	metered: boolean
}>()

const downloading = ref(false)
const { progress } = injectAppUpdateDownloadProgress()

function download() {
	emit('download')
	downloading.value = true
}

const messages = defineMessages({
	title: {
		id: 'app.update-toast.title',
		defaultMessage: 'Update available',
	},
	body: {
		id: 'app.update-toast.body',
		defaultMessage:
			'Modrinth App v{version} is ready to install! Reload to update now, or automatically when you close Modrinth App.',
	},
	reload: {
		id: 'app.update-toast.reload',
		defaultMessage: 'Reload',
	},
	download: {
		id: 'app.update-toast.download',
		defaultMessage: 'Download ({size})',
	},
	downloading: {
		id: 'app.update-toast.downloading',
		defaultMessage: 'Downloading...',
	},
	changelog: {
		id: 'app.update-toast.changelog',
		defaultMessage: 'Changelog',
	},
	meteredBody: {
		id: 'app.update-toast.body.metered',
		defaultMessage: `Modrinth App v{version} is available now! Since you're on a metered network, we didn't automatically download it.`,
	},
	downloadCompleteTitle: {
		id: 'app.update-toast.title.download-complete',
		defaultMessage: 'Download complete',
	},
	downloadedBody: {
		id: 'app.update-toast.body.download-complete',
		defaultMessage: `Modrinth App v{version} has finished downloading. Reload to update now, or automatically when you close Modrinth App.`,
	},
})
</script>
<template>
	<div
		class="grid grid-cols-[min-content] fixed card-shadow rounded-2xl top-[--top-bar-height] mt-6 right-6 p-4 z-10 bg-bg-raised border-divider border-solid border-[2px]"
		:class="{
			'download-complete': progress === 1,
		}"
	>
		<div class="flex min-w-[25rem] gap-4">
			<h2 class="whitespace-nowrap text-base text-contrast font-semibold m-0 grow">
				{{
					formatMessage(metered && progress === 1 ? messages.downloadCompleteTitle : messages.title)
				}}
			</h2>
			<ButtonStyled size="small" circular>
				<button v-tooltip="formatMessage(commonMessages.closeButton)" @click="emit('close')">
					<XIcon />
				</button>
			</ButtonStyled>
		</div>
		<p class="text-sm mt-2 mb-0">
			{{
				formatMessage(
					metered
						? progress === 1
							? messages.downloadedBody
							: messages.meteredBody
						: messages.body,
					{ version },
				)
			}}
		</p>
		<p
			v-if="metered && progress < 1"
			class="text-sm text-secondary mt-2 mb-0 flex items-center gap-1"
		>
			<template v-if="progress > 0">
				<ProgressBar :progress="progress" class="max-w-[unset]" />
			</template>
		</p>
		<div class="flex gap-2 mt-4">
			<ButtonStyled color="brand">
				<button v-if="metered && progress < 1" :disabled="downloading" @click="download">
					<SpinnerIcon v-if="downloading" class="animate-spin" />
					<DownloadIcon v-else />
					{{
						formatMessage(downloading ? messages.downloading : messages.download, {
							size: formatBytes(size ?? 0),
						})
					}}
				</button>
				<button v-else @click="emit('restart')">
					<RefreshCwIcon /> {{ formatMessage(messages.reload) }}
				</button>
			</ButtonStyled>
			<ButtonStyled>
				<a href="https://modrinth.com/news/changelog?filter=app">
					{{ formatMessage(messages.changelog) }} <ExternalIcon />
				</a>
			</ButtonStyled>
		</div>
	</div>
</template>
