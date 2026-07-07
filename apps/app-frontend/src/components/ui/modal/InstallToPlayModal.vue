<template>
	<NewModal ref="modal" :header="formatMessage(messages.installToPlay)" :closable="true">
		<div
			v-if="mode === 'server-project' && requiredContentProject"
			class="flex flex-col gap-6 max-w-[500px]"
		>
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
								<BulletDivider /> {{ formatMessage(messages.modCount, { count: modCount }) }}
							</template>
						</span>
					</div>
				</div>
			</div>
		</div>
		<div
			v-else-if="mode === 'shared-instance' && sharedInstance"
			class="flex flex-col gap-6 max-w-[500px]"
		>
			<Admonition type="warning" :header="formatMessage(messages.trustWarningHeader)">
				{{ formatMessage(messages.trustWarningDescription) }}
			</Admonition>

			<div class="flex flex-col gap-1">
				<div class="flex justify-between items-center">
					<span class="font-semibold text-contrast">{{
						formatMessage(messages.sharedInstance)
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
						:src="sharedInstance.preview.iconUrl"
						:alt="sharedInstance.preview.name"
						size="48px"
					/>
					<div class="flex flex-col gap-0.5">
						<span class="font-semibold text-contrast">{{ sharedInstance.preview.name }}</span>
						<span class="text-sm text-secondary">
							{{ sharedInstanceLoaderDisplay }} {{ sharedInstance.preview.gameVersion }}
							<template v-if="sharedInstance.preview.modCount">
								<BulletDivider />
								{{
									formatMessage(messages.modCount, {
										count: sharedInstance.preview.modCount,
									})
								}}
							</template>
						</span>
					</div>
				</div>
			</div>

			<div
				v-if="sharedInstance.preview.externalFileCount"
				class="flex items-center gap-2 text-orange"
			>
				<IssuesIcon class="h-5 w-5 flex-none" />
				<span>{{
					formatMessage(messages.externalFileWarning, {
						count: sharedInstance.preview.externalFileCount,
					})
				}}</span>
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
		:header="contentModalHeader"
		:modpack-name="contentModalName"
		:modpack-icon-url="contentModalIconUrl"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, EyeIcon, IssuesIcon, XIcon } from '@modrinth/assets'
import type { ContentItem } from '@modrinth/ui'
import {
	Admonition,
	Avatar,
	BulletDivider,
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
import type { SharedInstanceInstallPreview } from '@/helpers/install'
import { injectServerInstall } from '@/providers/server-install'

const modal = ref<InstanceType<typeof NewModal>>()
const mode = ref<'server-project' | 'shared-instance'>('server-project')
const modpackVersionId = ref<string | null>(null)
const modpackVersion = ref<Labrinth.Versions.v2.Version | null>(null)
const project = ref<Labrinth.Projects.v3.Project | null>(null)
const requiredContentProject = ref<Labrinth.Projects.v2.Project | null>(null)
const onInstallComplete = ref<() => void>(() => {})
const sharedInstance = ref<{
	preview: SharedInstanceInstallPreview
	invitedByUsername?: string | null
} | null>(null)
const onSharedInstanceInstall = ref<() => void | Promise<void>>(() => {})
const { formatMessage } = useVIntl()
const { installServerProject, startInstallingServer, stopInstallingServer } = injectServerInstall()

const usingCustomModpack = computed(() => {
	return requiredContentProject.value?.id === project.value?.id
})

const loaderDisplay = computed(() => {
	const loader = requiredContentProject.value?.loaders?.[0]
	if (!loader) return ''
	return formatLoader(formatMessage, loader)
})

const modCount = computed(() => modpackVersion.value?.dependencies?.length)
const sharedInstanceLoaderDisplay = computed(() => {
	const loader = sharedInstance.value?.preview.loader
	if (!loader) return ''
	return formatLoader(formatMessage, loader)
})
const contentModalName = computed(() =>
	mode.value === 'shared-instance'
		? (sharedInstance.value?.preview.name ?? '')
		: (project.value?.name ?? ''),
)
const contentModalIconUrl = computed(() =>
	mode.value === 'shared-instance'
		? (sharedInstance.value?.preview.iconUrl ?? undefined)
		: (project.value?.icon_url ?? undefined),
)
const contentModalHeader = computed(() =>
	mode.value === 'shared-instance' ? formatMessage(messages.sharedInstanceContent) : undefined,
)

type VersionDependency = Labrinth.Versions.v2.Dependency & {
	version_id?: string
}

async function fetchData(versionId: string) {
	// cache is making version null for some reason so bypassing for now
	modpackVersion.value = await get_version(versionId, 'bypass')

	if (modpackVersion.value?.project_id) {
		requiredContentProject.value = await get_project(modpackVersion.value.project_id, 'bypass')
	}
}

async function handleAccept() {
	hide()
	if (mode.value === 'shared-instance') {
		try {
			await onSharedInstanceInstall.value()
		} catch (error) {
			console.error('Failed to install shared instance from InstallToPlayModal:', error)
		}
		return
	}

	const serverProjectId = project.value?.id
	startInstallingServer(serverProjectId)
	try {
		await installServerProject(serverProjectId)
		onInstallComplete.value()
	} catch (error) {
		console.error('Failed to install server project from InstallToPlayModal:', error)
	} finally {
		stopInstallingServer(serverProjectId)
	}
}

function handleDecline() {
	hide()
}

const modpackContentModal = ref<InstanceType<typeof ModpackContentModal>>()

async function openViewContents() {
	if (mode.value === 'shared-instance') {
		await openSharedInstanceContents()
		return
	}

	modpackContentModal.value?.showLoading()
	try {
		// Ensure version data is available — the useQuery may not have resolved yet
		const versionId = modpackVersionId.value
		const version =
			modpackVersion.value ?? (versionId ? await get_version(versionId, 'must_revalidate') : null)

		const contentItems = await contentItemsFromDependencies(version?.dependencies ?? [])
		modpackContentModal.value?.show(contentItems)
	} catch (err) {
		console.error('Failed to load modpack contents:', err)
		modpackContentModal.value?.show([])
	}
}

async function openSharedInstanceContents() {
	const preview = sharedInstance.value?.preview
	if (!preview) return

	modpackContentModal.value?.showLoading()
	try {
		const contentItems: ContentItem[] = [
			...preview.externalFiles.map(sharedExternalFileContentItem),
			...(await sharedInstanceModpackContentItems(preview)),
			...(await contentItemsFromVersionIds(
				preview.contentVersionIds.filter((id) => id !== preview.modpackVersionId),
			)),
		]

		modpackContentModal.value?.show(contentItems)
	} catch (err) {
		console.error('Failed to load shared instance contents:', err)
		modpackContentModal.value?.show([])
	}
}

async function sharedInstanceModpackContentItems(preview: SharedInstanceInstallPreview) {
	if (!preview.modpackVersionId) return []

	const version = await get_version(preview.modpackVersionId, 'must_revalidate')
	return await contentItemsFromDependencies(version?.dependencies ?? [])
}

async function contentItemsFromDependencies(deps: Labrinth.Versions.v2.Dependency[]) {
	const dependencies = deps as VersionDependency[]
	const projectIds = unique(
		dependencies.map((dependency) => dependency.project_id).filter((id): id is string => !!id),
	)
	const versionIds = unique(
		dependencies.map((dependency) => dependency.version_id).filter((id): id is string => !!id),
	)

	const projects: Labrinth.Projects.v2.Project[] =
		projectIds.length > 0 ? await get_project_many(projectIds, 'must_revalidate') : []
	const versions: Labrinth.Versions.v2.Version[] =
		versionIds.length > 0 ? await get_version_many(versionIds, 'must_revalidate') : []
	const projectMap = new Map(projects.map((depProject) => [depProject.id, depProject]))
	const versionMap = new Map(versions.map((depVersion) => [depVersion.id, depVersion]))

	return dependencies.map((dependency): ContentItem => {
		const depProject = dependency.project_id ? projectMap.get(dependency.project_id) : null
		const depVersion = dependency.version_id ? versionMap.get(dependency.version_id) : null
		const fileName =
			depVersion?.files?.[0]?.filename ?? dependency.file_name ?? depProject?.title ?? 'Unknown'
		const external = !depProject && !depVersion

		return {
			id: dependency.version_id ?? dependency.project_id ?? fileName,
			file_name: fileName,
			project_type: depProject?.project_type ?? 'mod',
			has_update: false,
			update_version_id: null,
			external,
			project: {
				id: depProject?.id ?? dependency.project_id ?? fileName,
				slug: depProject?.slug ?? dependency.project_id ?? fileName,
				title: depProject?.title ?? dependency.file_name ?? fileName,
				icon_url: depProject?.icon_url ?? undefined,
			},
			...(depVersion
				? {
						version: {
							id: depVersion.id,
							file_name: depVersion.files?.[0]?.filename ?? dependency.file_name ?? fileName,
							version_number: depVersion.version_number ?? undefined,
							date_published: depVersion.date_published ?? undefined,
						},
					}
				: {}),
		}
	})
}

async function contentItemsFromVersionIds(versionIds: string[]) {
	const uniqueVersionIds = unique(versionIds)
	const versions: Labrinth.Versions.v2.Version[] =
		uniqueVersionIds.length > 0 ? await get_version_many(uniqueVersionIds, 'must_revalidate') : []
	const projectIds = unique(versions.map((version) => version.project_id).filter(Boolean))
	const projects: Labrinth.Projects.v2.Project[] =
		projectIds.length > 0 ? await get_project_many(projectIds, 'must_revalidate') : []
	const projectMap = new Map(projects.map((depProject) => [depProject.id, depProject]))

	return versions.map((version): ContentItem => {
		const depProject = projectMap.get(version.project_id)
		const fileName = version.files?.[0]?.filename ?? depProject?.title ?? version.name ?? 'Unknown'

		return {
			id: version.id,
			file_name: fileName,
			project_type: depProject?.project_type ?? 'mod',
			has_update: false,
			update_version_id: null,
			project: {
				id: depProject?.id ?? version.project_id,
				slug: depProject?.slug ?? version.project_id,
				title: depProject?.title ?? version.name,
				icon_url: depProject?.icon_url ?? undefined,
			},
			version: {
				id: version.id,
				file_name: fileName,
				version_number: version.version_number ?? undefined,
				date_published: version.date_published ?? undefined,
			},
		}
	})
}

function sharedExternalFileContentItem(
	file: SharedInstanceInstallPreview['externalFiles'][number],
): ContentItem {
	return {
		id: `external:${file.fileType}:${file.fileName}`,
		file_name: file.fileName,
		project_type: file.fileType,
		has_update: false,
		update_version_id: null,
		external: true,
		project: {
			id: file.fileName,
			slug: file.fileName,
			title: file.fileName,
			icon_url: null,
		},
	}
}

function unique<T>(values: T[]) {
	return Array.from(new Set(values))
}

async function show(
	projectVal: Labrinth.Projects.v3.Project,
	modpackVersionIdVal: string | null = null,
	callback: () => void = () => {},
	e?: MouseEvent,
) {
	mode.value = 'server-project'
	project.value = projectVal
	sharedInstance.value = null
	modpackVersionId.value = modpackVersionIdVal
	modpackVersion.value = null
	requiredContentProject.value = null
	onInstallComplete.value = callback
	onSharedInstanceInstall.value = () => {}

	if (modpackVersionIdVal) await fetchData(modpackVersionIdVal)

	hide_ads_window()
	modal.value?.show(e)
}

function showSharedInstance(
	instance: {
		preview: SharedInstanceInstallPreview
		invitedByUsername?: string | null
	},
	install: () => void | Promise<void>,
	e?: MouseEvent,
) {
	mode.value = 'shared-instance'
	project.value = null
	modpackVersionId.value = null
	modpackVersion.value = null
	requiredContentProject.value = null
	onInstallComplete.value = () => {}
	sharedInstance.value = instance
	onSharedInstanceInstall.value = install

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
	sharedInstanceContent: {
		id: 'app.modal.install-to-play.shared-instance-content',
		defaultMessage: 'Shared instance content',
	},
	trustWarningHeader: {
		id: 'app.modal.install-to-play.trust-warning-header',
		defaultMessage: 'Do you trust this user?',
	},
	trustWarningDescription: {
		id: 'app.modal.install-to-play.trust-warning-description',
		defaultMessage:
			'A shared instance will install files on your computer and may include content not from Modrinth.',
	},
	externalFileWarning: {
		id: 'app.modal.install-to-play.external-file-warning',
		defaultMessage:
			'{count, plural, one {This instance includes # file that is not from Modrinth.} other {This instance includes # files that are not from Modrinth.}}',
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

defineExpose({ show, showSharedInstance, hide })
</script>
