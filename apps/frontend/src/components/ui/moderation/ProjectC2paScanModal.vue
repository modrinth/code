<script setup lang="ts">
import { ExternalIcon } from '@modrinth/assets'
import { ButtonLink, NewModal } from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

import { fileDeclaresAi } from '~/helpers/c2pa.ts'

const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')
const isScanning = ref(false)

const fileUrl = ref<string | null>(null)
const totalFiles = ref(0)
const checkedFiles = ref(0)
const aiFiles = ref<string[]>([])

async function scanFile(url: string) {
	if (isScanning.value) return

	fileUrl.value = url
	isScanning.value = true
	totalFiles.value = 0
	checkedFiles.value = 0
	aiFiles.value = []

	try {
		const { BlobReader, ZipReader } = await import('@zip.js/zip.js')

		const response = await fetch(url)
		if (!response.ok) {
			throw new Error(`Failed to fetch file: ${response.statusText}`)
		}
		const blob = await response.blob()

		const reader = new ZipReader(new BlobReader(blob))
		const entries = await reader.getEntries()

		totalFiles.value = entries.length

		for (const entry of entries) {
			checkedFiles.value++

			if (entry.directory) continue
			if (!entry.filename.endsWith('.png')) continue

			const buffer = await entry.arrayBuffer()
			const isAiFile = await fileDeclaresAi(
				new File([buffer], entry.filename, { type: 'image/png' }),
			)

			if (isAiFile) {
				aiFiles.value.push(entry.filename)
			}
		}
	} finally {
		isScanning.value = false
	}
}

function createSlicerUrl(path: string) {
	const downloadUrl = new URL(fileUrl.value!)
	const fileName = downloadUrl.pathname.split('/').pop()

	return `https://slicer.run/?file=${encodeURIComponent(`${fileName}/${path}`)}&url=${encodeURIComponent(downloadUrl.toString())}`
}

function openC2paModal(url: string) {
	modalRef.value?.show()
	void scanFile(url)
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
				<div v-if="aiFiles.length === 0" class="flex items-center justify-center">
					<span class="rounded-xl bg-highlight-green px-4 py-1">No AI-generated files found.</span>
				</div>

				<div v-if="aiFiles.length > 0" class="flex flex-col gap-1">
					<span class="ml-2 text-xl font-semibold text-contrast"
						>AI-generated files ({{ aiFiles.length }})</span
					>
					<div
						v-for="file in aiFiles"
						:key="file"
						class="flex flex-row flex-wrap items-center justify-between gap-2 rounded-2xl bg-surface-2 p-4 font-semibold text-secondary"
					>
						{{ file }}
						<ButtonLink :href="createSlicerUrl(file)" target="_blank">
							Open <ExternalIcon />
						</ButtonLink>
					</div>
				</div>
			</div>
		</div>
	</NewModal>
</template>
