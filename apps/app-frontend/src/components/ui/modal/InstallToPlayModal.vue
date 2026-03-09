<template>
	<NewModal ref="modal" :header="formatMessage(messages.installToPlay)" :closable="true">
		<div v-if="requiredContentProject" class="flex flex-col gap-6 max-w-[500px]">
			<Admonition type="info" :header="formatMessage(messages.contentRequired)">
				{{ formatMessage(messages.serverRequiresMods) }}
			</Admonition>

			<div class="flex flex-col gap-1">
				<div class="flex justify-between items-center">
					<span class="font-semibold text-contrast">{{
						formatMessage(messages.requiredModpack)
					}}</span>

					<ButtonStyled type="transparent">
						<button @click="openViewContents">
							<EyeIcon />
							{{ formatMessage(messages.viewContents) }}
						</button>
					</ButtonStyled>
				</div>

				<div class="flex items-center gap-3 rounded-xl bg-surface-2 p-3">
					<Avatar
						:src="requiredContentProject.icon_url"
						:alt="requiredContentProject.title"
						size="48px"
					/>
					<div class="flex flex-col gap-0.5">
						<span class="font-semibold text-contrast">
							<template v-if="usingCustomModpack && modpackVersion">
								{{ modpackVersion.name }}
							</template>
							<template v-else>
								{{ requiredContentProject.title }}
							</template>
						</span>
						<span class="text-sm text-secondary">
							{{ loaderDisplay }} {{ requiredContentProject.game_versions?.[0] }}
							<template v-if="modCount">
								· {{ formatMessage(messages.modCount, { count: modCount }) }}
							</template>
						</span>
					</div>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled>
					<button @click="handleDecline">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button @click="handleAccept">
						<DownloadIcon />
						{{ formatMessage(messages.installButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>

	<ModpackContentModal
		ref="modpackContentModal"
		:modpack-name="project?.name ?? ''"
		:modpack-icon-url="project?.icon_url ?? undefined"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, EyeIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	formatLoader,
	ModpackContentModal,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { hide_ads_window, show_ads_window } from '@/helpers/ads'
import { get_project, get_project_many, get_version, get_version_many } from '@/helpers/cache.js'
import { installServerProject, useInstall } from '@/store/install.js'

import type { ContentItem } from '../../../../../../packages/ui/src/components/instances/types'

const modal = ref<InstanceType<typeof NewModal>>()
const modpackVersionId = ref<string | null>(null)
const modpackVersion = ref<Labrinth.Versions.v2.Version | null>(null)
const project = ref<Labrinth.Projects.v3.Project | null>(null)
const requiredContentProject = ref<Labrinth.Projects.v2.Project | null>(null)
const onInstallComplete = ref<() => void>(() => {})
const { formatMessage } = useVIntl()
const installStore = useInstall()

const usingCustomModpack = computed(() => {
	return requiredContentProject.value?.id === project.value?.id
})

const loaderDisplay = computed(() => {
	const loader = requiredContentProject.value?.loaders?.[0]
	if (!loader) return ''
	return formatLoader(formatMessage, loader)
})

const modCount = computed(() => modpackVersion.value?.dependencies?.length)

async function fetchData(versionId: string) {
	// cache is making version null for some reason so bypassing for now
	modpackVersion.value = await get_version(versionId, 'bypass')

	if (modpackVersion.value?.project_id) {
		requiredContentProject.value = await get_project(modpackVersion.value.project_id, 'bypass')
	}
}

async function handleAccept() {
	hide()
	const serverProjectId = project.value?.id
	installStore.startInstallingServer(serverProjectId)
	try {
		await installServerProject(serverProjectId)
		onInstallComplete.value()
	} catch (error) {
		console.error('Failed to install server project from InstallToPlayModal:', error)
	} finally {
		installStore.stopInstallingServer(serverProjectId)
	}
}

function handleDecline() {
	hide()
}

const modpackContentModal = ref<InstanceType<typeof ModpackContentModal>>()

async function openViewContents() {
	modpackContentModal.value?.showLoading()
	try {
		// Ensure version data is available — the useQuery may not have resolved yet
		const versionId = modpackVersionId.value
		const version =
			modpackVersion.value ?? (versionId ? await get_version(versionId, 'must_revalidate') : null)

		const deps = version?.dependencies ?? []

		const projectIds = deps
			.map((d: { project_id?: string }) => d.project_id)
			.filter((id: string | undefined): id is string => !!id)

		const versionIds = deps
			.map((d: { version_id?: string }) => d.version_id)
			.filter((id: string | undefined): id is string => !!id)

		const projects: Labrinth.Projects.v2.Project[] =
			projectIds.length > 0 ? await get_project_many(projectIds, 'must_revalidate') : []

		const versions: Labrinth.Versions.v2.Version[] =
			versionIds.length > 0 ? await get_version_many(versionIds, 'must_revalidate') : []

		const projectMap = new Map(projects.map((p: Labrinth.Projects.v2.Project) => [p.id, p]))

		const contentItems: ContentItem[] = deps.map(
			(dep: Labrinth.Versions.v2.Dependency): ContentItem => {
				const depProject = dep.project_id ? projectMap.get(dep.project_id) : null
				// @ts-expect-error - version_id is missing from the type for some reason
				const depVersion = dep.version_id
					? // @ts-expect-error - version_id is missing from the type for some reason
						versions.find((v: Labrinth.Versions.v2.Version) => v.id === dep.version_id)
					: null

				return {
					file_name: dep.file_name ?? depProject?.title ?? 'Unknown',
					project_type: depProject?.project_type ?? 'mod',
					has_update: false,
					update_version_id: null,
					project: {
						id: depProject?.id ?? dep.project_id ?? dep.file_name ?? 'unknown',
						slug: depProject?.slug ?? dep.project_id ?? 'unknown',
						title: depProject?.title ?? dep.file_name ?? 'Unknown',
						icon_url: depProject?.icon_url ?? undefined,
					},
					...(depVersion
						? {
								version: {
									id: depVersion.id,
									file_name: depVersion.files?.[0]?.filename ?? dep.file_name,
									version_number: depVersion.version_number ?? undefined,
									date_published: depVersion.date_published ?? undefined,
								},
							}
						: {}),
				}
			},
		)
		modpackContentModal.value?.show(contentItems)
	} catch (err) {
		console.error('Failed to load modpack contents:', err)
		modpackContentModal.value?.show([])
	}
}

async function show(
	projectVal: Labrinth.Projects.v3.Project,
	modpackVersionIdVal: string | null = null,
	callback: () => void = () => {},
	e?: MouseEvent,
) {
	project.value = projectVal
	modpackVersionId.value = modpackVersionIdVal
	modpackVersion.value = null
	requiredContentProject.value = null
	onInstallComplete.value = callback

	if (modpackVersionIdVal) await fetchData(modpackVersionIdVal)

	hide_ads_window()
	modal.value?.show(e)
}

function hide() {
	modal.value?.hide()
	show_ads_window()
}

const messages = defineMessages({
	installToPlay: {
		id: 'app.modal.install-to-play.header',
		defaultMessage: 'Install to play',
	},
	sharedServerInstance: {
		id: 'app.modal.install-to-play.shared-server-instance',
		defaultMessage: 'Shared server instance',
	},
	contentRequired: {
		id: 'app.modal.install-to-play.content-required',
		defaultMessage: 'Content required',
	},
	serverRequiresMods: {
		id: 'app.modal.install-to-play.server-requires-mods',
		defaultMessage:
			'This server requires mods to play. Click Install to set up the required files from Modrinth, then launch directly into the server.',
	},
	requiredModpack: {
		id: 'app.modal.install-to-play.required-modpack',
		defaultMessage: 'Required modpack',
	},
	sharedInstance: {
		id: 'app.modal.install-to-play.shared-instance',
		defaultMessage: 'Shared instance',
	},
	modCount: {
		id: 'app.modal.install-to-play.mod-count',
		defaultMessage: '{count, plural, one {# mod} other {# mods}}',
	},
	installButton: {
		id: 'app.modal.install-to-play.install-button',
		defaultMessage: 'Install',
	},
	viewContents: {
		id: 'app.modal.install-to-play.view-contents',
		defaultMessage: 'View contents',
	},
})

defineExpose({ show, hide })
</script>
