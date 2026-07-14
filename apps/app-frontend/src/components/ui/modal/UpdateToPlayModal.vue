<template>
	<ContentDiffModal
		ref="diffModal"
		:header="formatMessage(messages.updateToPlay)"
		:description="
			instance ? formatMessage(messages.updateRequiredDescription, { name: instance.name }) : ''
		"
		:diffs="normalizedDiffs"
		:version-date="versionDate"
		:show-external-warnings="showExternalWarnings"
		:confirm-label="formatMessage(commonMessages.updateButton)"
		:confirm-icon="DownloadIcon"
		:removed-label="formatMessage(messages.removed)"
		@confirm="handleUpdate"
		@cancel="handleDecline"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon } from '@modrinth/assets'
import {
	commonMessages,
	type ContentDiffItem,
	ContentDiffModal,
	defineMessages,
	useVIntl,
} from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, ref } from 'vue'

import { get_project_many, get_version, get_version_many } from '@/helpers/cache.js'
import { wait_for_install_job } from '@/helpers/install'
import { update_managed_modrinth_version } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { injectServerInstall } from '@/providers/server-install'

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
	fileName?: string
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
const { startInstallingServer, stopInstallingServer } = injectServerInstall()
type UpdateCompleteCallback = () => void | Promise<void>

defineProps<{
	showExternalWarnings?: boolean
}>()

const diffModal = ref<InstanceType<typeof ContentDiffModal>>()
const instance = ref<GameInstance | null>(null)
const onUpdateComplete = ref<UpdateCompleteCallback>(() => {})
const diffs = ref<DependencyDiff[]>([])
const modpackVersionId = ref<string | null>(null)
const modpackVersion = ref<Version | null>(null)

const normalizedDiffs = computed<ContentDiffItem[]>(() =>
	diffs.value.map((diff) => ({
		type: diff.type,
		external: Boolean(diff.fileName && !diff.project),
		projectName: diff.project?.title,
		fileName: diff.fileName,
		currentVersionName: diff.currentVersion?.version_number,
		newVersionName: diff.newVersion?.version_number,
	})),
)

const versionDate = computed(() =>
	modpackVersion.value?.date_published
		? dayjs(modpackVersion.value.date_published).format('MMMM D, YYYY')
		: undefined,
)

async function computeDependencyDiffs(
	currentDeps: Dependency[],
	latestDeps: Dependency[],
): Promise<DependencyDiff[]> {
	console.log('Computing dependency diffs', { currentDeps, latestDeps })

	// Separate deps with project_id from file_name-only deps
	const currentWithProject = currentDeps.filter((d) => d.project_id)
	const latestWithProject = latestDeps.filter((d) => d.project_id)
	const currentFileOnly = currentDeps.filter((d) => !d.project_id && d.file_name)
	const latestFileOnly = latestDeps.filter((d) => !d.project_id && d.file_name)

	const currentByProject = new Map<string, Dependency>(
		currentWithProject.map((d) => [d.project_id!, d]),
	)
	const latestByProject = new Map<string, Dependency>(
		latestWithProject.map((d) => [d.project_id!, d]),
	)
	const currentFilenames = new Set(currentFileOnly.map((d) => d.file_name!))
	const latestFilenames = new Set(latestFileOnly.map((d) => d.file_name!))

	const diffs: DependencyDiff[] = []

	// Find added and updated dependencies (by project_id)
	latestByProject.forEach((latestDep, projectId) => {
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

	// Find removed dependencies (by project_id)
	currentByProject.forEach((currentDep, projectId) => {
		if (!latestByProject.has(projectId)) {
			diffs.push({
				type: 'removed',
				project_id: projectId,
				currentVersionId: currentDep.version_id,
			})
		}
	})

	// Find added/removed file_name-only dependencies
	// ideally in future, this should use the hash of the file instead of filename, but since version dependencies don't include file hashes, we'll use filename as a best effort approach
	for (const fileName of latestFilenames) {
		if (!currentFilenames.has(fileName)) {
			diffs.push({ type: 'added', project_id: '', newVersionId: '' as string, fileName })
		}
	}
	for (const fileName of currentFilenames) {
		if (!latestFilenames.has(fileName)) {
			diffs.push({ type: 'removed', project_id: '', fileName })
		}
	}

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

	const mappedDiffs = diffs
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
			const aExternal = Boolean(a.fileName && !a.project)
			const bExternal = Boolean(b.fileName && !b.project)
			if (aExternal !== bExternal) return aExternal ? -1 : 1

			const typeOrder = { added: 0, updated: 1, removed: 2 }
			const typeCompare = typeOrder[a.type] - typeOrder[b.type]
			if (typeCompare !== 0) return typeCompare

			const aDate = a.newVersion?.date_published || a.currentVersion?.date_published || ''
			const bDate = b.newVersion?.date_published || b.currentVersion?.date_published || ''
			return dayjs(bDate).valueOf() - dayjs(aDate).valueOf()
		})
		.filter((d) => d.project || d.fileName) // filter out any diffs that couldn't be matched to a project or file
	return mappedDiffs
}

async function checkUpdateAvailable(inst: GameInstance): Promise<DependencyDiff[] | null> {
	if (!inst.link) return null
	if (!modpackVersionId.value || !inst.link.version_id) return null

	try {
		// For server projects, link.project_id is the server project but
		// link.version_id references a content modpack version from a different project.
		// Detect this by comparing the version's project_id with link.project_id.
		modpackVersion.value = await get_version(modpackVersionId.value, 'bypass')
		const instanceModpackVersion = await get_version(inst.link.version_id, 'bypass')

		// Compute dependency diffs between current and latest version
		if (instanceModpackVersion && modpackVersion.value) {
			return await computeDependencyDiffs(
				instanceModpackVersion.dependencies || [],
				modpackVersion.value.dependencies || [],
			)
		}
	} catch (error) {
		console.error('Error checking for updates:', error)
		return null
	}
	return null
}

async function handleUpdate() {
	hide()
	const serverProjectId = instance.value?.link?.project_id
	if (serverProjectId) startInstallingServer(serverProjectId)
	try {
		if (modpackVersionId.value && instance.value) {
			const job = await update_managed_modrinth_version(instance.value.id, modpackVersionId.value)
			await wait_for_install_job(job.job_id)
			await onUpdateComplete.value()
		}
	} catch (error) {
		console.error('Error updating instance:', error)
	} finally {
		if (serverProjectId) stopInstallingServer(serverProjectId)
	}
}

function handleDecline() {
	hide()
}

function show(
	instanceVal: GameInstance,
	modpackVersionIdVal: string | null = null,
	callback: UpdateCompleteCallback = () => {},
	e?: MouseEvent,
) {
	instance.value = instanceVal
	modpackVersionId.value = modpackVersionIdVal
	modpackVersion.value = null
	diffs.value = []
	onUpdateComplete.value = callback
	diffModal.value?.show(e)
	void checkUpdateAvailable(instanceVal).then((result) => {
		if (instance.value?.id === instanceVal.id && modpackVersionId.value === modpackVersionIdVal) {
			diffs.value = result || []
		}
	})
}

function hide() {
	diffModal.value?.hide()
}

const messages = defineMessages({
	updateToPlay: {
		id: 'app.modal.update-to-play.header',
		defaultMessage: 'Update to play',
	},
	updateRequiredDescription: {
		id: 'app.modal.update-to-play.update-required-description',
		defaultMessage:
			'An update is required to play {name}. Please update to the latest version to launch the game.',
	},
	removed: {
		id: 'app.modal.update-to-play.removed',
		defaultMessage: 'Removed',
	},
})

const hasUpdate = computed(() => {
	if (!instance.value?.link) return false
	return modpackVersionId.value != null && modpackVersionId.value !== instance.value.link.version_id
})

defineExpose({ show, hide, hasUpdate })
</script>
