<script setup lang="ts">
import { ChevronRightIcon, FileIcon, LeftArrowIcon, XIcon } from '@modrinth/assets'
import { ButtonStyled, NewModal, injectNotificationManager } from '@modrinth/ui'
import type { SearchResult as Project, Version, VersionFile } from '@modrinth/utils'
import { useDebounceFn } from '@vueuse/core'
import { computed, nextTick, ref, useTemplateRef } from 'vue'

type Step = 'project' | 'version' | 'file' | 'review'

const steps: Step[] = ['project', 'version', 'file', 'review']
const titles: Record<Step, string> = {
	project: 'Select project',
	version: 'Select version',
	file: 'Select file',
	review: 'Submit for analysis',
}

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const notifications = injectNotificationManager()

const selectedProject = ref<Project | null>(null)
const selectedVersion = ref<Version | null>(null)
const selectedFile = ref<VersionFile | null>(null)
const fileId = ref<number | null>(null)

const projectQuery = ref('')
const projects = ref<Project[]>([])
const versions = ref<Version[]>([])
const files = ref<VersionFile[]>([])

const searchLoading = ref(false)
const loadingVersions = ref(false)
const submitting = ref(false)

const currentStep = ref<Step>('project')
const currentStepIndex = computed(() => steps.indexOf(currentStep.value))
const previousStep = computed(() => steps[currentStepIndex.value - 1])
const nextStep = computed(() => steps[currentStepIndex.value + 1])

const canProceed = computed(() => {
	switch (currentStep.value) {
		case 'project':
			return !!selectedProject.value
		case 'version':
			return !!selectedVersion.value
		case 'file':
			return !!selectedFile.value
		case 'review':
			return !!fileId.value && !submitting.value
		default:
			return false
	}
})

const searchProjects = useDebounceFn(async () => {
	if (!projectQuery.value || projectQuery.value.trim().length < 2) {
		projects.value = []
		return
	}
	searchLoading.value = true
	try {
		const res = (await useBaseFetch(
			`search?query=${encodeURIComponent(projectQuery.value.trim())}&limit=8`,
		)) as any
		projects.value = (res?.hits ?? []) as Project[]
	} catch (e: any) {
		notifications.addNotification({
			title: 'Error',
			text: 'Failed to search for projects',
			type: 'error',
		})
	} finally {
		searchLoading.value = false
	}
}, 250)

async function loadVersions(projectId: string) {
	loadingVersions.value = true
	try {
		const res = (await useBaseFetch(`project/${projectId}/version`)) as Version[]
		versions.value = (res ?? []).sort(
			(a, b) =>
				new Date(b.date_published ?? 0).getTime() - new Date(a.date_published ?? 0).getTime(),
		)
	} catch (e: any) {
		versions.value = []
		notifications.addNotification({
			title: 'Error',
			text: 'Failed to load project versions',
			type: 'error',
		})
	} finally {
		loadingVersions.value = false
	}
}

async function chooseProject(project: Project) {
	selectedProject.value = project

	selectedVersion.value = null
	selectedFile.value = null
	files.value = []
	fileId.value = null

	await loadVersions(project.project_id || project.id)
	await setStep('version', true)
}

function chooseVersion(version: Version) {
	selectedVersion.value = version
	selectedFile.value = null
	files.value = version.files ?? []
	fileId.value = null
	setStep('file', true)
}

function chooseFile(file: VersionFile) {
	selectedFile.value = file
	fileId.value = Number(file.hashes.sha512)
	setStep('review', true)
}

async function beforeProceed(target: Step | undefined) {
	switch (target) {
		case 'version':
			if (!selectedProject.value) return false

			if (versions.value.length === 0) {
				await loadVersions(selectedProject.value.id || selectedProject.value.id)
			}
			return true
		case 'file':
			return !!selectedVersion.value
		case 'review':
			return !!selectedFile.value
		default:
			return true
	}
}

async function afterProceed(_target: Step | undefined) {}

async function setStep(target: Step | undefined, skipValidation = false) {
	if (!target) {
		await confirmRunDelphi()
		return
	}
	if (!skipValidation && !canProceed.value) return
	if (!(await beforeProceed(target))) return

	currentStep.value = target
	await nextTick()
	await afterProceed(target)
}

function resetState() {
	projectQuery.value = ''
	projects.value = []
	selectedProject.value = null

	versions.value = []
	selectedVersion.value = null

	files.value = []
	selectedFile.value = null

	fileId.value = null
	currentStep.value = 'project'
}

function show() {
	resetState()
	modal.value?.show()
}

defineExpose({ show })

const emit = defineEmits<{
	(
		e: 'started',
		payload: { fileId: number; project?: Project; version?: Version; file?: VersionFile },
	): void
	(e: 'hide'): void
}>()

async function confirmRunDelphi() {
	if (!fileId.value) {
		notifications.addNotification({
			title: 'Error',
			text: 'Please select a file to analyze',
			type: 'error',
		})
		return
	}

	submitting.value = true
	try {
		await useBaseFetch('delphi/run', {
			method: 'POST',
			body: { file_id: fileId.value },
			internal: true,
		})

		notifications.addNotification({
			title: 'Success',
			text: 'Delphi analysis started',
			type: 'success',
		})
		emit('started', {
			fileId: fileId.value,
			project: selectedProject.value ?? undefined,
			version: selectedVersion.value ?? undefined,
			file: selectedFile.value ?? undefined,
		})
		modal.value?.hide()
	} catch (error: any) {
		notifications.addNotification({
			title: 'Error',
			text: error?.message || 'Failed to start Delphi analysis',
			type: 'error',
		})
	} finally {
		submitting.value = false
	}
}

function versionTypeColor(type?: string) {
	switch (type) {
		case 'release':
			return 'green'
		case 'beta':
			return 'orange'
		case 'alpha':
			return 'red'
		default:
			return 'secondary'
	}
}
function formatVersionType(type?: string) {
	if (!type) return 'Unknown'
	return type.charAt(0).toUpperCase() + type.slice(1)
}
function formatFileSize(bytes?: number) {
	if (!bytes && bytes !== 0) return 'Unknown size'
	const units = ['B', 'KB', 'MB', 'GB']
	let size = Number(bytes)
	let i = 0
	while (size >= 1024 && i < units.length - 1) {
		size /= 1024
		i++
	}
	return `${size.toFixed(1)} ${units[i]}`
}
function formatRelativeTime(date?: string) {
	if (!date) return ''
	const d = new Date(date)
	const diffSec = Math.floor((Date.now() - d.getTime()) / 1000)
	if (diffSec < 60) return 'just now'
	if (diffSec < 3600) return `${Math.floor(diffSec / 60)} minutes ago`
	if (diffSec < 86400) return `${Math.floor(diffSec / 3600)} hours ago`
	if (diffSec < 2592000) return `${Math.floor(diffSec / 86400)} days ago`
	return d.toLocaleDateString()
}
</script>

<template>
	<NewModal ref="modal" @hide="$emit('hide')">
		<template #title>
			<div class="flex items-center gap-1 font-bold text-secondary">
				<template v-for="(title, idx) in steps" :key="title">
					<button
						v-if="idx < currentStepIndex"
						class="bg-transparent p-0 font-bold text-secondary active:scale-95"
						@click="setStep(title, true)"
					>
						{{ titles[title] }}
					</button>
					<span v-else :class="{ 'text-contrast': idx === currentStepIndex }">
						{{ titles[title] }}
					</span>
					<ChevronRightIcon
						v-if="idx < steps.length - 1"
						class="h-5 w-5 text-secondary"
						stroke-width="3"
					/>
				</template>
			</div>
		</template>

		<div class="w-[40rem] max-w-full space-y-4">
			<div v-if="currentStep === 'project'" class="space-y-3">
				<label for="project-search" class="text-sm font-medium text-contrast">
					Search for a project <span class="text-brand-red">*</span>
				</label>
				<div class="relative">
					<input
						id="project-search"
						v-model="projectQuery"
						type="text"
						class="border-border bg-raised w-full rounded-lg border px-3 py-2 text-sm placeholder:text-secondary focus:border-brand focus:outline-none"
						placeholder="Enter project name..."
						autocomplete="off"
						@input="searchProjects"
					/>
					<div v-if="searchLoading" class="absolute right-3 top-2.5">
						<div
							class="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"
						/>
					</div>
				</div>

				<div
					v-if="projects.length"
					class="border-border max-h-64 overflow-y-auto rounded-lg border"
				>
					<div
						v-for="p in projects"
						:key="p.id"
						class="border-border flex cursor-pointer items-center gap-3 border-b p-3 hover:bg-button-bg"
						@click="chooseProject(p)"
					>
						<img
							:src="p.icon_url || 'https://cdn.modrinth.com/placeholder.png'"
							:alt="p.title"
							class="h-10 w-10 rounded-md object-cover"
						/>
						<div>
							<div class="font-medium text-contrast">{{ p.title }}</div>
							<div class="text-xs text-secondary">{{ p.author }}</div>
						</div>
					</div>
				</div>
				<div v-else-if="projectQuery && !searchLoading" class="mt-2 text-center text-secondary">
					No projects found
				</div>
			</div>

			<div v-else-if="currentStep === 'version'" class="space-y-3">
				<div class="mb-2 flex items-center gap-2">
					<div class="flex items-center gap-2">
						<img
							:src="selectedProject?.icon_url || 'https://cdn.modrinth.com/placeholder.png'"
							:alt="selectedProject?.title || ''"
							class="h-6 w-6 rounded-md object-cover"
						/>
						<span class="font-medium text-contrast">{{ selectedProject?.title }}</span>
					</div>
				</div>

				<label class="mt-2 text-sm font-medium text-contrast">
					Select a version <span class="text-brand-red">*</span>
				</label>
				<div v-if="loadingVersions" class="py-4 text-center">
					<div
						class="mx-auto h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent"
					/>
				</div>
				<div v-else-if="!versions.length" class="py-4 text-center text-secondary">
					No versions found
				</div>
				<div v-else class="border-border max-h-64 overflow-y-auto rounded-lg border">
					<div
						v-for="v in versions"
						:key="v.id"
						class="border-border flex cursor-pointer items-center justify-between border-b p-3 hover:bg-button-bg"
						@click="chooseVersion(v)"
					>
						<div>
							<div class="font-medium text-contrast">{{ v.version_number }}</div>
							<div class="flex flex-wrap gap-1 text-xs text-secondary">
								<span>{{ formatVersionType(v.version_type) }}</span>
								<span>•</span>
								<span>{{ formatRelativeTime(v.date_published) }}</span>
							</div>
						</div>
						{{ formatVersionType(v.version_type) }}
					</div>
				</div>
			</div>

			<div v-else-if="currentStep === 'file'" class="space-y-3">
				<div class="mb-2 flex items-center gap-2">
					<div class="flex items-center gap-2">
						<img
							:src="selectedProject?.icon_url || 'https://cdn.modrinth.com/placeholder.png'"
							:alt="selectedProject?.title || ''"
							class="h-6 w-6 rounded-md object-cover"
						/>
						<span class="font-medium text-contrast">{{ selectedProject?.title }}</span>
						<span class="text-secondary">→</span>
						<span class="font-medium text-contrast">{{ selectedVersion?.version_number }}</span>
					</div>
				</div>

				<span class="text-sm font-medium text-contrast">
					Select a file to analyze <span class="text-brand-red">*</span>
				</span>

				<div
					v-if="(files?.length ?? 0) > 0"
					class="divide-border border-border max-h-64 divide-y overflow-y-auto rounded-lg border"
				>
					<div
						v-for="f in files"
						:key="String(f.id)"
						class="flex cursor-pointer items-center justify-between p-3 hover:bg-button-bg"
						@click="chooseFile(f)"
					>
						<div class="flex items-center gap-2">
							<FileIcon class="h-4 w-4 text-secondary" />
							<div>
								<div class="font-medium text-contrast">{{ f.filename }}</div>
								<div class="text-xs text-secondary">{{ formatFileSize(f.size) }}</div>
							</div>
						</div>
						Primary
					</div>
				</div>
				<div v-else class="py-4 text-center text-secondary">No files in this version</div>
			</div>

			<div v-else-if="currentStep === 'review'" class="space-y-4">
				<div class="flex items-center gap-2">
					<ButtonStyled size="small" circular>
						<button @click="setStep(previousStep, true)"><LeftArrowIcon class="h-4 w-4" /></button>
					</ButtonStyled>
					<div class="flex items-center gap-2">
						<img
							:src="selectedProject?.icon_url || 'https://cdn.modrinth.com/placeholder.png'"
							:alt="selectedProject?.title || ''"
							class="h-6 w-6 rounded-md object-cover"
						/>
						<span class="font-medium text-contrast">{{ selectedProject?.title }}</span>
						<span class="text-secondary">→</span>
						<span class="font-medium text-contrast">{{ selectedVersion?.version_number }}</span>
					</div>
				</div>

				<div class="border-border rounded-lg border p-4">
					<div class="space-y-2">
						<div class="flex items-center gap-2">
							<FileIcon class="h-4 w-4 text-secondary" />
							<span class="font-medium text-contrast">{{ selectedFile?.filename }}</span>
						</div>
						<div class="text-sm text-secondary">{{ formatFileSize(selectedFile?.size) }}</div>
						<div class="flex items-center gap-2 text-sm">
							<span class="text-secondary">File ID:</span>
							<code class="rounded bg-button-bg px-2 py-0.5 font-mono text-sm">
								{{ fileId ?? '' }}
							</code>
						</div>
					</div>
				</div>
			</div>
		</div>

		<div class="mt-4 flex justify-between gap-2">
			<ButtonStyled>
				<button v-if="previousStep" @click="setStep(previousStep, true)">
					<LeftArrowIcon /> Back
				</button>
				<button v-else @click="modal?.hide()"><XIcon /> Cancel</button>
			</ButtonStyled>

			<ButtonStyled color="brand" v-if="currentStep === 'review'">
				<button :disabled="!canProceed" @click="setStep(nextStep)">
					<template>
						<span v-if="submitting">Starting…</span>
						<span v-else>Run Analysis</span>
					</template>
				</button>
			</ButtonStyled>
		</div>
	</NewModal>
</template>
