<script setup lang="ts">
import { CoffeeIcon, ExternalIcon } from '@modrinth/assets'
import {
	Button,
	ButtonLink,
	defineMessages,
	injectNotificationManager,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

import { fileDeclaresAi } from '~/helpers/c2pa.ts'
import { injectReviewSlicer, slicerFileName } from '~/providers/project-review/review-slicer'

const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')
const slicer = injectReviewSlicer(null)
const isScanning = ref(false)
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	noAiFiles: {
		id: 'moderation.c2pa-scan.no-ai-files',
		defaultMessage: 'No AI-generated files found.',
	},
	scanFailed: {
		id: 'moderation.c2pa-scan.failed',
		defaultMessage: 'Failed to scan for AI-generated files.',
	},
})

const totalFiles = ref(0)
const checkedFiles = ref(0)
const aiFiles = ref<{ url: string; path: string }[]>([])
const scanFailed = ref(false)

async function scanFiles(urls: string[]) {
	if (isScanning.value || !urls.length) return

	isScanning.value = true
	totalFiles.value = 0
	checkedFiles.value = 0
	aiFiles.value = []
	scanFailed.value = false
	modalRef.value?.show()

	try {
		const { BlobReader, ZipReader } = await import('@zip.js/zip.js')

		for (const url of new Set(urls)) {
			try {
				const response = await fetch(url)
				if (!response.ok) {
					throw new Error(`Failed to fetch file: ${response.statusText}`)
				}

				const reader = new ZipReader(new BlobReader(await response.blob()))
				try {
					const entries = await reader.getEntries()
					totalFiles.value += entries.length

					for (const entry of entries) {
						checkedFiles.value++

						if (entry.directory || !entry.filename.toLowerCase().endsWith('.png')) continue

						const buffer = await entry.arrayBuffer()
						if (await fileDeclaresAi(new File([buffer], entry.filename, { type: 'image/png' }))) {
							aiFiles.value.push({ url, path: entry.filename })
						}
					}
				} finally {
					await reader.close()
				}
			} catch (error) {
				scanFailed.value = true
				addNotification({
					type: 'error',
					title: formatMessage(messages.scanFailed),
					text: `${url}: ${error instanceof Error ? error.message : String(error)}`,
				})
			}
		}

		if (!aiFiles.value.length && !scanFailed.value) {
			addNotification({
				type: 'success',
				title: formatMessage(messages.noAiFiles),
			})
		}
	} catch (error) {
		scanFailed.value = true
		addNotification({
			type: 'error',
			title: formatMessage(messages.scanFailed),
			text: error instanceof Error ? error.message : String(error),
		})
	} finally {
		isScanning.value = false
	}
}

function createSlicerUrl(file: { url: string; path: string }) {
	return `https://slicer.run/?file=${encodeURIComponent(`${slicerFileName(file.url)}/${file.path}`)}&url=${encodeURIComponent(file.url)}`
}

function openInSlicerTab(file: { url: string; path: string }) {
	if (!slicer) return
	slicer.open([file.url], `${slicerFileName(file.url)}/${file.path}`)
	hide()
}

function openC2paModal(url: string | string[]) {
	void scanFiles(Array.isArray(url) ? url : [url])
}

function hide() {
	modalRef.value?.hide()
}

defineExpose({ openC2paModal, hide })
</script>

<template>
	<NewModal ref="modalRef" width="40vw" :disable-close="isScanning">
		<template #title>
			<span class="text-2xl font-semibold text-contrast">C2PA Scan Info</span>
		</template>

		<div class="w-full">
			<div v-if="isScanning" class="flex items-center justify-center">
				<span class="rounded-xl bg-highlight-blue px-4 py-1"
					>Scanning {{ checkedFiles }}/{{ totalFiles }} files.</span
				>
			</div>
			<div v-else>
				<div v-if="scanFailed" class="text-red">
					{{ formatMessage(messages.scanFailed) }}
				</div>
				<div v-if="aiFiles.length === 0 && !scanFailed" class="flex items-center justify-center">
					<span class="rounded-xl bg-highlight-green px-4 py-1">No AI-generated files found.</span>
				</div>

				<div v-if="aiFiles.length > 0" class="flex flex-col gap-1">
					<span class="ml-2 text-xl font-semibold text-contrast"
						>AI-generated files ({{ aiFiles.length }})</span
					>
					<div
						v-for="file in aiFiles"
						:key="`${file.url}/${file.path}`"
						class="flex flex-row flex-wrap items-center justify-between gap-2 rounded-2xl bg-surface-2 p-4 font-semibold text-secondary"
					>
						{{ file.path }}
						<span class="break-all text-xs text-secondary">{{ file.url }}</span>
						<Button v-if="slicer" @click="openInSlicerTab(file)"> Open <CoffeeIcon /> </Button>
						<ButtonLink v-else :href="createSlicerUrl(file)" target="_blank">
							Open <ExternalIcon />
						</ButtonLink>
					</div>
				</div>
			</div>
		</div>
	</NewModal>
</template>
