<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.installToPlay)"
		:closable="true"
		:on-hide="show_ads_window"
		max-width="640px"
		width="640px"
	>
		<div v-if="requiredContentProject" class="flex w-full flex-col gap-6">
			<p class="m-0 text-primary">
				{{ formatMessage(messages.inviteWarning) }}
			</p>

			<div class="flex flex-col gap-2.5">
				<div class="flex items-center justify-between">
					<span class="font-semibold text-contrast">
						{{ formatMessage(messages.sharedInstance) }}
					</span>

					<ButtonStyled type="transparent">
						<button @click="openViewContents">
							<EyeIcon />
							{{ formatMessage(messages.viewContents) }}
						</button>
					</ButtonStyled>
				</div>

				<div class="flex items-center gap-3 rounded-2xl bg-surface-2 p-3">
					<Avatar
						:src="requiredContentProject.icon_url"
						:alt="requiredContentProject.title"
						size="56px"
						no-shadow
						class="!rounded-2xl"
					/>
					<div class="flex min-w-0 flex-col gap-0.5">
						<span class="truncate font-semibold text-contrast">
							<template v-if="usingCustomModpack && modpackVersion">
								{{ modpackVersion.name }}
							</template>
							<template v-else>
								{{ requiredContentProject.title }}
							</template>
						</span>
						<span class="truncate text-sm font-medium text-secondary">
							{{ loaderDisplay }} {{ requiredContentProject.game_versions?.[0] }}
							<template v-if="modCount">
								· {{ formatMessage(messages.modCount, { count: modCount }) }}
							</template>
						</span>
					</div>
				</div>
			</div>

			<Admonition
				v-if="hasExternalFiles"
				type="warning"
				:header="formatMessage(messages.unknownFilesWarning)"
			>
				{{ formatMessage(messages.unknownFilesDescription) }}
			</Admonition>

			<div v-if="hasExternalFiles" class="relative w-full">
				<div
					ref="externalFileTable"
					class="max-h-[242px] overflow-y-auto rounded-2xl"
					@scroll="checkTableScrollState"
				>
					<Table
						:columns="externalFileColumns"
						:data="externalFileRows"
						row-key="id"
						virtualized
						:virtual-row-height="48"
						class="shadow-sm"
					>
						<template #cell-name="{ value }">
							<span class="block truncate" :title="String(value)">{{ value }}</span>
						</template>
					</Table>
				</div>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-2"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-2"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTableTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 bg-gradient-to-b from-bg-raised to-transparent"
					/>
				</Transition>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-2"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-2"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTableBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 bg-gradient-to-t from-bg-raised to-transparent"
					/>
				</Transition>
			</div>

			<p v-if="hasExternalFiles" class="m-0 text-primary">
				{{ formatMessage(messages.reviewedFiles) }}
			</p>

			<div class="flex w-full items-center justify-between gap-2">
				<ButtonStyled type="transparent" color="red">
					<button @click="handleReport">
						<ReportIcon />
						{{ formatMessage(commonMessages.reportButton) }}
					</button>
				</ButtonStyled>

				<div class="flex items-center gap-2">
					<template v-if="hasExternalFiles">
						<ButtonStyled type="transparent" color="orange">
							<button @click="handleAccept">
								{{ formatMessage(messages.installAnyway) }}
							</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button @click="handleDecline">
								<BanIcon />
								{{ formatMessage(messages.dontInstall) }}
							</button>
						</ButtonStyled>
					</template>
					<template v-else>
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
					</template>
				</div>
			</div>
		</div>
	</NewModal>

	<ModpackContentModal
		ref="modpackContentModal"
		:modpack-name="project?.name ?? ''"
		:modpack-icon-url="project?.icon_url ?? undefined"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { BanIcon, DownloadIcon, EyeIcon, ReportIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	ButtonStyled,
	commonMessages,
	type ContentItem,
	defineMessages,
	formatLoader,
	ModpackContentModal,
	NewModal,
	Table,
	type TableColumn,
	useScrollIndicator,
	useVIntl,
} from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, nextTick, ref } from 'vue'

import { hide_ads_window, show_ads_window } from '@/helpers/ads'
import { get_project, get_project_many, get_version, get_version_many } from '@/helpers/cache.js'
import { injectServerInstall } from '@/providers/server-install'

type ExternalFileColumn = 'name'
type ExternalFileRow = {
	id: string
	name: string
}

const modal = ref<InstanceType<typeof NewModal>>()
const modpackVersionId = ref<string | null>(null)
const modpackVersion = ref<Labrinth.Versions.v2.Version | null>(null)
const project = ref<Labrinth.Projects.v3.Project | null>(null)
const requiredContentProject = ref<Labrinth.Projects.v2.Project | null>(null)
const externalFiles = ref<string[]>([])
const externalFileTable = ref<HTMLElement | null>(null)
const onInstallComplete = ref<() => void>(() => {})
const { formatMessage } = useVIntl()

const props = defineProps<{
	showExternalWarnings?: boolean
}>()

const { installServerProject, startInstallingServer, stopInstallingServer } = injectServerInstall()
const {
	showTopFade: showTableTopFade,
	showBottomFade: showTableBottomFade,
	checkScrollState: checkTableScrollState,
	forceCheck: forceCheckTableScroll,
} = useScrollIndicator(externalFileTable)

const usingCustomModpack = computed(() => {
	return requiredContentProject.value?.id === project.value?.id
})

const loaderDisplay = computed(() => {
	const loader = requiredContentProject.value?.loaders?.[0]
	if (!loader) return ''
	return formatLoader(formatMessage, loader)
})

const modCount = computed(() => modpackVersion.value?.dependencies?.length)
const hasExternalFiles = computed(
	() => Boolean(props.showExternalWarnings) && externalFiles.value.length > 0,
)
const externalFileRows = computed<ExternalFileRow[]>(() =>
	externalFiles.value.map((name, index) => ({
		id: `${index}-${name}`,
		name,
	})),
)

async function fetchData(versionId: string) {
	// cache is making version null for some reason so bypassing for now
	const version = await get_version(versionId, 'bypass')
	modpackVersion.value = version

	if (version?.project_id) {
		requiredContentProject.value = await get_project(version.project_id, 'bypass')
		externalFiles.value = [
			...new Set(
				(version.dependencies ?? [])
					.filter(
						(dependency) =>
							dependency.dependency_type === 'embedded' &&
							!dependency.project_id &&
							!dependency.version_id &&
							dependency.file_name,
					)
					.flatMap((dependency) => (dependency.file_name ? [dependency.file_name] : [])),
			),
		].sort((left, right) => left.localeCompare(right))
	}
}

async function handleAccept() {
	hide()
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

function handleReport() {
	if (project.value?.id) {
		openUrl(`https://modrinth.com/report?item=project&itemID=${project.value.id}`)
	}
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
	externalFiles.value = []
	onInstallComplete.value = callback

	if (modpackVersionIdVal) await fetchData(modpackVersionIdVal)

	hide_ads_window()
	modal.value?.show(e)
	await nextTick()
	forceCheckTableScroll()
}

function hide() {
	modal.value?.hide()
}

const messages = defineMessages({
	installToPlay: {
		id: 'app.modal.install-to-play.header',
		defaultMessage: 'Install to play',
	},
	inviteWarning: {
		id: 'app.modal.install-to-play.invite-warning',
		defaultMessage:
			'This invite was created by another Modrinth user, not Modrinth. Only accept invites from people you trust.',
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
	unknownFilesWarning: {
		id: 'app.modal.install-to-play.unknown-files-warning',
		defaultMessage: 'Unknown files warning',
	},
	unknownFilesDescription: {
		id: 'app.modal.install-to-play.unknown-files-description',
		defaultMessage:
			'This server modpack contains files that aren’t published on Modrinth. We strongly recommend only installing files from sources you trust.',
	},
	unrecognizedFiles: {
		id: 'app.modal.install-to-play.unrecognized-files',
		defaultMessage: 'Unrecognized files',
	},
	reviewedFiles: {
		id: 'app.modal.install-to-play.reviewed-files',
		defaultMessage:
			'A file is only reviewed if it’s published to Modrinth, regardless of its file format (including .mrpack).',
	},
	installAnyway: {
		id: 'app.modal.install-to-play.install-anyway',
		defaultMessage: 'Install anyway',
	},
	dontInstall: {
		id: 'app.modal.install-to-play.dont-install',
		defaultMessage: 'Dont install',
	},
})

const externalFileColumns = computed<TableColumn<ExternalFileColumn>[]>(() => [
	{
		key: 'name',
		label: formatMessage(messages.unrecognizedFiles),
		cellClass: '!h-12',
	},
])

defineExpose({ show, hide })
</script>
