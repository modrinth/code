<template>
	<div class="normal-page no-sidebar">
		<h1>File lookup</h1>
		<div class="normal-page__content">
			<div class="card flex flex-col gap-3">
				<div
					class="border-highlight-gray hover:bg-button-hover relative flex h-32 cursor-pointer flex-col items-center justify-center rounded-xl border-2 border-dashed bg-button-bg p-8 transition-colors"
					@click="triggerFileInput"
					@drop.prevent="handleDrop"
					@dragover.prevent
					@dragenter.prevent
				>
					<p
						class="mx-auto mb-0 flex items-center gap-2 text-center text-lg font-bold text-primary"
					>
						<UploadIcon /> Select file to lookup
					</p>
					<p class="mx-auto mt-0 text-center text-sm text-secondary">
						Drag and drop or click here to browse
					</p>
					<input ref="fileInput" type="file" class="hidden" @change="handleFileSelect" />
				</div>

				<template v-if="selectedFile">
					<div class="flex items-center gap-2 text-sm text-secondary">
						<FileIcon class="h-4 w-4" />
						<span>{{ selectedFile.name }} ({{ formatBytes(selectedFile.size) }})</span>
					</div>

					<div v-if="loadingHash" class="flex items-center gap-2 text-sm text-secondary">
						<SpinnerIcon class="h-4 w-4 animate-spin" />
						Calculating hashes...
					</div>
					<div v-if="loadingLookup" class="flex items-center gap-2 text-sm text-secondary">
						<SpinnerIcon class="h-4 w-4 animate-spin" />
						Looking up file on Modrinth...
					</div>

					<template v-if="fileHashes">
						<h3 class="mb-0 text-lg font-extrabold text-contrast">File hashes:</h3>
						<div class="flex flex-col gap-2">
							<span class="text-xs text-secondary">SHA512:</span>
							<CopyCode :text="fileHashes.sha512" />
							<span class="mt-1 text-xs text-secondary">SHA256:</span>
							<CopyCode :text="fileHashes.sha256" />
							<span class="mt-1 text-xs text-secondary">SHA1:</span>
							<CopyCode :text="fileHashes.sha1" />
						</div>
					</template>
				</template>

				<template v-if="lookupResult">
					<h3 class="mb-0 text-lg font-extrabold text-contrast">Modrinth project:</h3>
					<nuxt-link
						class="flex w-fit items-center gap-2 text-lg font-semibold text-contrast hover:underline"
						target="_blank"
						:to="`/project/${lookupResult.projectId}`"
					>
						<Avatar :src="lookupResult.iconUrl" alt="" size="48px" />
						{{ lookupResult.name }}
					</nuxt-link>
					<CopyCode :text="lookupResult.projectId" />
					<h3 class="mb-0 text-lg font-extrabold text-contrast">Modrinth version:</h3>
					<nuxt-link
						class="text-blue hover:underline"
						:to="`/project/${lookupResult.projectId}/version/${lookupResult.versionId}`"
						target="_blank"
					>
						Version {{ lookupResult.versionNumber }}
					</nuxt-link>
					<CopyCode :text="lookupResult.versionId" />
				</template>

				<Admonition v-if="lookupError" type="critical" header="Lookup failed">
					{{ lookupError }}
				</Admonition>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { FileIcon, SpinnerIcon, UploadIcon } from '@modrinth/assets'
import { Admonition, Avatar, CopyCode, injectNotificationManager } from '@modrinth/ui'
import { formatBytes, type Project, type Version } from '@modrinth/utils'

const { addNotification } = injectNotificationManager()

const fileInput = ref<HTMLInputElement>()
const selectedFile = ref<File | null>(null)
const fileHashes = ref<{
	sha512: string
	sha256: string
	sha1: string
} | null>(null)
const loadingHash = ref(false)
const loadingLookup = ref(false)

const lookupResult = ref<{
	projectId: string
	versionId: string
	name: string
	versionNumber: string
	iconUrl?: string | undefined
}>()
const lookupError = ref<string>('')

function triggerFileInput() {
	fileInput.value?.click()
}

function handleFileSelect(event: Event) {
	const target = event.target as HTMLInputElement
	if (target.files && target.files.length > 0) {
		processFile(target.files[0])
	}
}

function handleDrop(event: DragEvent) {
	event.preventDefault()
	if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
		processFile(event.dataTransfer.files[0])
	}
}

async function processFile(file: File) {
	selectedFile.value = file
	fileHashes.value = null
	lookupResult.value = undefined
	lookupError.value = ''

	await calculateHashesAndLookup(file)
}

function formatHashBuffer(buffer: ArrayBuffer) {
	return Array.from(new Uint8Array(buffer))
		.map((b) => b.toString(16).padStart(2, '0'))
		.join('')
}

async function calculateHashesAndLookup(file: File): Promise<void> {
	loadingHash.value = true
	loadingLookup.value = true

	try {
		const buffer = await file.arrayBuffer()

		const [sha512, sha256, sha1] = await Promise.all([
			crypto.subtle.digest('SHA-512', buffer).then(formatHashBuffer),
			crypto.subtle.digest('SHA-256', buffer).then(formatHashBuffer),
			crypto.subtle.digest('SHA-1', buffer).then(formatHashBuffer),
		])

		fileHashes.value = { sha512, sha256, sha1 }

		await lookupFile(sha512)
	} catch (error) {
		console.error('Error calculating hashes:', error)
		addNotification({
			title: 'Hash calculation failed',
			text: 'Failed to calculate file hashes.',
			type: 'error',
		})
	} finally {
		loadingHash.value = false
		loadingLookup.value = false
	}
}

async function lookupFile(hash: string): Promise<void> {
	if (!hash) {
		return
	}

	try {
		const version = (await useBaseFetch(`version_file/${hash}?algorithm=sha512`, {
			method: 'GET',
		})) as Version

		if (version) {
			const project = (await useBaseFetch(`project/${version.project_id}`, {
				method: 'GET',
			})) as Project

			lookupResult.value = {
				projectId: project.id,
				versionId: version.id,
				versionNumber: version.version_number,
				name: project.title,
				iconUrl: project.icon_url,
			}
		}
	} catch (error: any) {
		if (error.status === 404) {
			lookupError.value = `File not found on Modrinth across projects you have access to.`
		} else {
			lookupError.value = error.data?.description || 'Failed to lookup file.'
		}
	}
}
</script>
