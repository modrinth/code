<template>
	<NewModal ref="modal" header="Update to play" :closable="true" no-padding>
		<div class="max-w-[500px]">
			<div class="flex flex-col gap-4 p-4">
				<Admonition type="warning" header="Update required">
					An update is required to play {{ instance.name }}. Please update to the latest version to
					launch the game.
				</Admonition>

				<div class="flex flex-col gap-2">
					<span v-if="latestVersion?.date_published" class="text-contrast font-semibold">{{
						new Date(latestVersion.date_published).toLocaleDateString('en-US', {
							year: 'numeric',
							month: 'long',
							day: 'numeric',
						})
					}}</span>
					<div v-if="diffs.length" class="flex gap-2">
						<div
							v-if="diffs.filter((d) => d.type === 'updated').length"
							class="flex gap-1 items-center"
						>
							<RefreshCwIcon />
							{{ diffs.filter((d) => d.type === 'updated').length }} updated
						</div>
						<div
							v-if="diffs.filter((d) => d.type === 'added').length"
							class="flex gap-1 items-center"
						>
							<PlusIcon />
							{{ diffs.filter((d) => d.type === 'added').length }} added
						</div>
						<div
							v-if="diffs.filter((d) => d.type === 'removed').length"
							class="flex gap-1 items-center"
						>
							<MinusIcon />
							{{ diffs.filter((d) => d.type === 'removed').length }} removed
						</div>
					</div>
				</div>
			</div>
			<div v-if="diffs.length" class="flex flex-col bg-surface-2 p-4 max-h-[272px] overflow-y-auto">
				<div
					v-for="diff in diffs"
					:key="diff.project_id"
					class="grid grid-cols-[auto_1fr_1fr_1fr] items-center min-h-10 h-10 gap-2"
				>
					<div class="flex flex-col justify-between items-center">
						<div class="w-[1px] h-2"></div>
						<PlusIcon v-if="diff.type === 'added'" />
						<MinusIcon v-else-if="diff.type === 'removed'" />
						<RefreshCwIcon v-else />
						<div class="bg-surface-5 w-[1px] h-2 relative top-1"></div>
					</div>

					<div class="flex gap-1 col-span-2">
						<span class="text-sm capitalize">{{ diff.type }}</span>
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
						Report
					</button>
				</ButtonStyled>
				<div class="flex gap-2">
					<ButtonStyled>
						<button @click="handleDecline">
							<XIcon />
							Decline
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button @click="handleUpdate">
							<CheckIcon />
							Accept
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, MinusIcon, PlusIcon, RefreshCwIcon, ReportIcon, XIcon } from '@modrinth/assets'
import { Admonition, ButtonStyled, NewModal } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import dayjs from 'dayjs'
import { onMounted, ref } from 'vue'

import { get_project, get_project_many, get_version_many } from '@/helpers/cache.js'
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

const { instance } = defineProps<{
	instance: GameInstance
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const diffs = ref<DependencyDiff[]>([])
const latestVersionId = ref<string | null>(null)
const latestVersion = ref<Version | null>(null)

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
		get_project_many(allProjectIds, 'must_revalidate'),
		get_version_many(allVersionIds, 'must_revalidate'),
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
			const typeOrder = { added: 0, removed: 1, updated: 2 }
			const typeCompare = typeOrder[a.type] - typeOrder[b.type]
			if (typeCompare !== 0) return typeCompare

			const aDate = a.newVersion?.date_published || a.currentVersion?.date_published || ''
			const bDate = b.newVersion?.date_published || b.currentVersion?.date_published || ''
			return dayjs(bDate).valueOf() - dayjs(aDate).valueOf()
		})
}

async function checkUpdateAvailable(instance: GameInstance): Promise<DependencyDiff[] | null> {
	if (!instance.linked_data) return null

	try {
		const project = await get_project(instance.linked_data.project_id, 'must_revalidate')
		if (!project || !project.versions || project.versions.length === 0) {
			return null
		}

		const versions = await get_version_many(project.versions, 'must_revalidate')
		const sortedVersions = versions.sort(
			(a: { date_published: string }, b: { date_published: string }) =>
				dayjs(b.date_published).valueOf() - dayjs(a.date_published).valueOf(),
		)

		latestVersion.value = sortedVersions[0]
		latestVersionId.value = latestVersion.value?.id || null

		const currentVersionId = instance.linked_data.version_id
		const currentVersion = versions.find((v: { id: string }) => v.id === currentVersionId)

		// Compute dependency diffs between current and latest version
		if (currentVersion && latestVersion.value) {
			return await computeDependencyDiffs(
				currentVersion.dependencies || [],
				latestVersion.value.dependencies || [],
			)
		}
	} catch (error) {
		return null
	}
	return null
}

onMounted(async () => {
	const result = await checkUpdateAvailable(instance)
	diffs.value = result || []
})

async function handleUpdate() {
	hide()
	try {
		if (latestVersionId.value) {
			await update_managed_modrinth_version(instance.path, latestVersionId.value)
		}
	} catch (error) {
		console.error('Error updating instance:', error)
	}
}

function handleReport() {
	if (instance.linked_data?.project_id) {
		openUrl(`https://modrinth.com/report?item=project&itemID=${instance.linked_data.project_id}`)
	}
}

function handleDecline() {
	hide()
}

function show(e?: MouseEvent) {
	modal.value?.show(e)
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
