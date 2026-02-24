<template>
	<NewModal ref="modal" :header="formatMessage(messages.updateToPlay)" :closable="true" no-padding>
		<div v-if="instance" class="max-w-[500px]">
			<div class="flex flex-col gap-4 p-4">
				<Admonition type="info" :header="formatMessage(messages.updateRequired)">
					{{ formatMessage(messages.updateRequiredDescription, { name: instance.name }) }}
				</Admonition>

				<div v-if="diffs.length" class="flex flex-col gap-2">
					<span v-if="publishedDate" class="text-contrast font-semibold">{{
						formatMessage(messages.publishedDate, { date: publishedDate })
					}}</span>
					<div class="flex gap-2">
						<div v-if="removedCount" class="flex gap-1 items-center">
							<MinusIcon />
							{{ formatMessage(messages.removedCount, { count: removedCount }) }}
						</div>
						<div v-if="addedCount" class="flex gap-1 items-center">
							<PlusIcon />
							{{ formatMessage(messages.addedCount, { count: addedCount }) }}
						</div>
						<div v-if="updatedCount" class="flex gap-1 items-center">
							<RefreshCwIcon />
							{{ formatMessage(messages.updatedCount, { count: updatedCount }) }}
						</div>
					</div>
				</div>
			</div>
			<div
				v-if="diffs.length"
				class="flex flex-col bg-surface-2 p-4 max-h-[272px] overflow-y-auto border-t border-b border-r-0 border-l-0 border-solid border-surface-5"
			>
				<div
					v-for="(diff, index) in diffs"
					:key="diff.project_id"
					class="grid grid-cols-[auto_1fr_1fr_1fr] items-center min-h-10 h-10 gap-2"
				>
					<div class="flex flex-col justify-between items-center">
						<div class="w-[1px] h-2"></div>
						<PlusIcon v-if="diff.type === 'added'" />
						<MinusIcon v-else-if="diff.type === 'removed'" />
						<RefreshCwIcon v-else />
						<div
							:class="index === diffs.length - 1 ? 'bg-transparent' : 'bg-surface-5'"
							class="w-[1px] h-2 relative top-1"
						></div>
					</div>

					<div class="flex gap-1 col-span-2">
						<span class="text-sm">{{ formatMessage(diffTypeMessages[diff.type]) }}</span>
						<span
							v-if="diff.project"
							v-tooltip="diff.project.title"
							class="text-sm text-contrast font-medium truncate"
						>
							{{ diff.project.title }}
						</span>
					</div>

					<span
						v-if="getFilename(diff.newVersion) || getFilename(diff.currentVersion)"
						v-tooltip="getFilename(diff.newVersion) || getFilename(diff.currentVersion)"
						class="text-xs truncate text-right"
					>
						{{ getFilename(diff.newVersion) || getFilename(diff.currentVersion) }}
					</span>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-between gap-2">
				<ButtonStyled color="red" type="transparent">
					<button @click="handleReport">
						<ReportIcon />
						{{ formatMessage(commonMessages.reportButton) }}
					</button>
				</ButtonStyled>
				<div class="flex gap-2">
					<ButtonStyled>
						<button @click="handleDecline">
							<XIcon />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button @click="handleUpdate">
							<DownloadIcon />
							{{ formatMessage(commonMessages.updateButton) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	DownloadIcon,
	MinusIcon,
	PlusIcon,
	RefreshCwIcon,
	ReportIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	commonMessages,
	defineMessages,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import dayjs from 'dayjs'
import { computed, ref, watch } from 'vue'

import { get_project_many, get_version, get_version_many } from '@/helpers/cache.js'
import { update_managed_modrinth_version } from '@/helpers/profile'
import type { GameInstance } from '@/helpers/types'

type Dependency = Labrinth.Versions.v3.Dependency
type Version = Labrinth.Versions.v2.Version

interface BaseDiff {
	project_id: string
	project?: {
		title: string
		icon_url?: string
		slug: string
	}
	currentVersionId?: string
	newVersionId?: string
	currentVersion?: Version
	newVersion?: Version
}
interface AddedDiff extends BaseDiff {
	type: 'added'
	newVersionId: string
}
interface RemovedDiff extends BaseDiff {
	type: 'removed'
}
interface UpdatedDiff extends BaseDiff {
	type: 'updated'
	currentVersionId: string
	newVersionId: string
}
type DependencyDiff = AddedDiff | RemovedDiff | UpdatedDiff

type ProjectInfo = {
	id: string
	title: string
	icon_url?: string
	slug: string
}

const { formatMessage } = useVIntl()

const modal = ref<InstanceType<typeof NewModal>>()
const instance = ref<GameInstance | null>(null)
const modpackVersionId = ref<string | null>(null)
const onUpdateComplete = ref<() => void>(() => {})
const diffs = ref<DependencyDiff[]>([])
const latestVersionId = ref<string | null>(null)
const latestVersion = ref<Version | null>(null)

const removedCount = computed(() => diffs.value.filter((d) => d.type === 'removed').length)
const addedCount = computed(() => diffs.value.filter((d) => d.type === 'added').length)
const updatedCount = computed(() => diffs.value.filter((d) => d.type === 'updated').length)
const publishedDate = computed(() =>
	latestVersion.value?.date_published ? new Date(latestVersion.value.date_published) : null,
)

function getFilename(version?: Version): string | undefined {
	return version?.files.find((f) => f.primary)?.filename
}

async function computeDependencyDiffs(
	currentDeps: Dependency[],
	latestDeps: Dependency[],
): Promise<DependencyDiff[]> {
	const currentByProject = new Map<string, Dependency>(
		currentDeps.map((d) => [d.project_id || '', d]),
	)
	const latestByProject = new Map<string, Dependency>(
		latestDeps.map((d) => [d.project_id || '', d]),
	)

	const diffs: DependencyDiff[] = []

	// Find added and updated dependencies
	latestByProject.forEach((latestDep, projectId) => {
		if (!projectId) return
		const currentDep = currentByProject.get(projectId)
		if (!currentDep && latestDep.version_id) {
			diffs.push({ type: 'added', project_id: projectId, newVersionId: latestDep.version_id })
		} else if (
			currentDep?.version_id &&
			latestDep?.version_id &&
			currentDep?.version_id !== latestDep.version_id
		) {
			diffs.push({
				type: 'updated',
				project_id: projectId,
				currentVersionId: currentDep.version_id,
				newVersionId: latestDep.version_id,
			})
		}
	})

	// Find removed dependencies
	currentByProject.forEach((currentDep, projectId) => {
		if (!projectId) return
		if (!latestByProject.has(projectId)) {
			diffs.push({
				type: 'removed',
				project_id: projectId,
				currentVersionId: currentDep.version_id,
			})
		}
	})

	// Fetch projects and versions of diffs
	const allProjectIds = [...new Set(diffs.map((d) => d.project_id).filter(Boolean))]
	const allVersionIds = [
		...new Set(
			[...diffs.map((d) => d.newVersionId), ...diffs.map((d) => d.currentVersionId)].filter(
				Boolean,
			),
		),
	] as string[]
	const [projects, versions] = await Promise.all([
		get_project_many(allProjectIds, 'bypass'),
		get_version_many(allVersionIds, 'bypass'),
	])

	const projectMap = new Map<string, ProjectInfo>(projects.map((p: ProjectInfo) => [p.id, p]))
	const versionMap = new Map<string, Version>(versions.map((v: Version) => [v.id, v]))

	return diffs
		.map((diff) => {
			const project = projectMap.get(diff.project_id)
			return {
				...diff,
				project: project
					? { title: project.title, icon_url: project.icon_url, slug: project.slug }
					: undefined,
				currentVersion: diff.currentVersionId ? versionMap.get(diff.currentVersionId) : undefined,
				newVersion: diff.newVersionId ? versionMap.get(diff.newVersionId) : undefined,
			}
		})
		.sort((a, b) => {
			const typeOrder = { removed: 0, added: 1, updated: 2 }
			const typeCompare = typeOrder[a.type] - typeOrder[b.type]
			if (typeCompare !== 0) return typeCompare

			const aDate = a.newVersion?.date_published || a.currentVersion?.date_published || ''
			const bDate = b.newVersion?.date_published || b.currentVersion?.date_published || ''
			return dayjs(bDate).valueOf() - dayjs(aDate).valueOf()
		})
}

async function checkUpdateAvailable(inst: GameInstance): Promise<DependencyDiff[] | null> {
	if (!inst.linked_data) return null

	try {
		// For server projects, linked_data.project_id is the server project but
		// linked_data.version_id references a content modpack version from a different project.
		// Detect this by comparing the version's project_id with linked_data.project_id.
		const modpackVersion = await get_version(modpackVersionId.value, 'bypass')
		const instanceModpackVersion = await get_version(inst.linked_data.version_id, 'bypass')

		// Compute dependency diffs between current and latest version
		if (instanceModpackVersion && modpackVersion) {
			return await computeDependencyDiffs(
				modpackVersion.dependencies || [],
				instanceModpackVersion.dependencies || [],
			)
		}
	} catch (error) {
		console.error('Error checking for updates:', error)
		return null
	}
	return null
}

watch(
	() => instance.value,
	async (newInstance) => {
		if (!newInstance) return
		const result = await checkUpdateAvailable(newInstance)
		diffs.value = result || []
	},
	{ immediate: true, deep: true },
)

async function handleUpdate() {
	hide()
	try {
		if (latestVersionId.value && instance.value) {
			await update_managed_modrinth_version(instance.value.path, latestVersionId.value)
			onUpdateComplete.value()
		}
	} catch (error) {
		console.error('Error updating instance:', error)
	}
}

function handleReport() {
	if (instance.value?.linked_data?.project_id) {
		openUrl(
			`https://modrinth.com/report?item=project&itemID=${instance.value.linked_data.project_id}`,
		)
	}
}

function handleDecline() {
	hide()
}

function show(
	instanceVal: GameInstance,
	modpackVersionIdVal: string | null = null,
	callback: () => void = () => {},
	e?: MouseEvent,
) {
	instance.value = instanceVal
	modpackVersionId.value = modpackVersionIdVal
	onUpdateComplete.value = callback
	modal.value?.show(e)
}

function hide() {
	modal.value?.hide()
}

const messages = defineMessages({
	updateToPlay: {
		id: 'app.modal.update-to-play.header',
		defaultMessage: 'Update to play',
	},
	updateRequired: {
		id: 'app.modal.update-to-play.update-required',
		defaultMessage: 'Update required',
	},
	updateRequiredDescription: {
		id: 'app.modal.update-to-play.update-required-description',
		defaultMessage:
			'An update is required to play {name}. Please update to the latest version to launch the game.',
	},
	publishedDate: {
		id: 'app.modal.update-to-play.published-date',
		defaultMessage: '{date, date, long}',
	},
	removedCount: {
		id: 'app.modal.update-to-play.removed-count',
		defaultMessage: '{count} removed',
	},
	addedCount: {
		id: 'app.modal.update-to-play.added-count',
		defaultMessage: '{count} added',
	},
	updatedCount: {
		id: 'app.modal.update-to-play.updated-count',
		defaultMessage: '{count} updated',
	},
})

const diffTypeMessages = defineMessages({
	added: {
		id: 'app.modal.update-to-play.diff-type.added',
		defaultMessage: 'Added',
	},
	removed: {
		id: 'app.modal.update-to-play.diff-type.removed',
		defaultMessage: 'Removed',
	},
	updated: {
		id: 'app.modal.update-to-play.diff-type.updated',
		defaultMessage: 'Updated',
	},
})

const hasUpdate = computed(() => {
	if (!instance.value?.linked_data) return false
	return (
		latestVersionId.value != null && latestVersionId.value !== instance.value.linked_data.version_id
	)
})

defineExpose({ show, hide, hasUpdate })
</script>
