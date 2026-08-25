<template>
	<section class="overflow-visible">
		<!-- Loading state -->
		<div v-if="showVersionsLoadingState" class="flex items-center justify-center gap-2 py-8">
			<SpinnerIcon class="animate-spin" />
			<span>Loading versions...</span>
		</div>

		<template v-else>
			<CreateProjectVersionModal
				v-if="currentMember"
				ref="create-project-version-modal"
			></CreateProjectVersionModal>

			<ProjectC2paScanModal ref="project-c2pa-scan-modal"></ProjectC2paScanModal>

			<ConfirmModal
				v-if="currentMember"
				ref="deleteVersionModal"
				title="Are you sure you want to delete this version?"
				description="This will remove this version forever (like really forever)."
				:has-to-type="false"
				proceed-label="Delete"
				@proceed="deleteVersion()"
			/>

			<ProjectPageVersions
				v-if="versions?.length"
				:project="project"
				:versions="versions"
				:show-files="flags.showVersionFilesInTable"
				:show-environment-column="flags.showVersionEnvironmentColumn"
				:current-member="!!currentMember"
				:loaders="tags.loaders"
				:game-versions="tags.gameVersions"
				:base-id="baseDropdownId"
				:version-link="
					(version) =>
						`/${project.project_type}/${
							project.slug ? project.slug : project.id
						}/version/${encodeURI(version.displayUrlEnding ? version.displayUrlEnding : version.id)}`
				"
				:open-modal="currentMember ? () => handleOpenCreateVersionModal() : undefined"
			>
				<template #actions="{ version }">
					<ButtonLink
						v-if="getPrimaryFile(version)"
						v-tooltip="`Download`"
						type="quiet"
						:href="createDownloadUrl(version)"
						:download="getPrimaryFile(version)?.filename"
						class="!w-9 !rounded-full !px-0 hover:!bg-button-bg [&>svg]:!text-green"
						aria-label="Download"
						@click="emit('onDownload')"
					>
						<DownloadIcon aria-hidden="true" />
					</ButtonLink>
					<ButtonLink
						v-if="
							!!getPrimaryFile(version) &&
							isStaff(auth.user) &&
							modSettings.get(moderationSettings.General.SlicerButtonInVersions)
						"
						v-tooltip="`Open in Slicer`"
						type="quiet"
						target="_blank"
						:href="`https://slicer.run/?url=${encodeURIComponent(createDownloadUrl(version))}`"
						class="!w-9 !rounded-full !px-0 hover:!bg-button-bg"
						aria-label="Open in Slicer"
					>
						<ExternalIcon aria-hidden="true" />
					</ButtonLink>
					<TeleportOverflowMenu
						v-if="currentMember"
						type="quiet"
						label="Edit version"
						tooltip="Edit version"
						class="hover:!bg-button-bg"
						:options="[
							{
								id: 'edit-metadata',
								label: 'Edit metadata',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'metadata'),
							},
							{
								id: 'edit-details',
								label: 'Edit details',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'add-details'),
							},
							{
								id: 'edit-files',
								label: 'Edit files',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'add-files'),
							},
						]"
					>
						<EditIcon aria-hidden="true" />
						<template #edit-files>
							<FileIcon aria-hidden="true" />
							Edit files
						</template>
						<template #edit-details>
							<InfoIcon aria-hidden="true" />
							Edit details
						</template>
						<template #edit-metadata>
							<BoxIcon aria-hidden="true" />
							Edit metadata
						</template>
					</TeleportOverflowMenu>
					<TeleportOverflowMenu
						type="quiet"
						label="More options"
						tooltip="More options"
						class="hover:!bg-button-bg"
						:options="[
							{
								id: 'download',
								label: 'Download',
								type: 'link',
								tone: 'brand',
								hoverFilled: true,
								href: createDownloadUrl(version),
								download: getPrimaryFile(version)?.filename,
								shown: !!getPrimaryFile(version),
							},
							{
								id: 'new-tab',
								label: 'Open in new tab',
								type: 'link',
								href: `/${project.project_type}/${
									project.slug ? project.slug : project.id
								}/version/${encodeURI(version.displayUrlEnding ? version.displayUrlEnding : version.id)}`,
								target: '_blank',
							},
							{
								id: 'copy-link',
								label: 'Copy link',
								action: () =>
									copyToClipboard(
										`https://modrinth.com/${project.project_type}/${
											project.slug ? project.slug : project.id
										}/version/${encodeURI(version.displayUrlEnding ? version.displayUrlEnding : version.id)}`,
									),
							},
							{
								id: 'share',
								label: 'Share',
								action: () => {},
								shown: false,
							},
							{
								id: 'report',
								label: 'Report',
								tone: 'red',
								hoverFilled: true,
								action: () =>
									auth.user ? reportVersion(version.id) : navigateTo(getSignInRouteObj(route)),
								shown: !currentMember,
							},
							{ type: 'divider', shown: isStaff(auth.user) },
							{
								id: 'view-c2pa-info',
								label: 'View C2PA info',
								tone: 'orange',
								action: () => projectC2paScanModal.openC2paModal(createDownloadUrl(version)),
								shown: isStaff(auth.user),
							},
							{ type: 'divider', shown: currentMember || flags.developerMode },
							{
								id: 'copy-id',
								label: 'Copy ID',
								action: () => {
									copyToClipboard(version.id)
								},
								shown: currentMember || flags.developerMode,
							},
							{
								id: 'copy-maven',
								label: 'Copy Maven coordinates',
								action: () => {
									copyToClipboard(`maven.modrinth:${project.slug}:${version.id}`)
								},
								shown: flags.developerMode,
							},
							{ type: 'divider', shown: !!currentMember },
							{
								id: 'edit-metadata',
								label: 'Edit metadata',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'metadata'),
								shown: !!currentMember,
							},
							{
								id: 'edit-details',
								label: 'Edit details',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'add-details'),
								shown: !!currentMember,
							},
							{
								id: 'edit-files',
								label: 'Edit files',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'add-files'),
								shown: !!currentMember,
							},
							{
								id: 'delete',
								label: 'Delete',
								tone: 'red',
								hoverFilled: true,
								action: () => {
									selectedVersion = version.id
									deleteVersionModal?.show()
								},
								shown: !!currentMember,
							},
						]"
						@select="$event.id === 'download' && emit('onDownload')"
					>
						<MoreVerticalIcon aria-hidden="true" />
						<template #download>
							<DownloadIcon aria-hidden="true" />
							Download
						</template>
						<template #new-tab>
							<ExternalIcon aria-hidden="true" />
							Open in new tab
						</template>
						<template #copy-link>
							<LinkIcon aria-hidden="true" />
							Copy link
						</template>
						<template #share>
							<ShareIcon aria-hidden="true" />
							Share
						</template>
						<template #report>
							<ReportIcon aria-hidden="true" />
							Report
						</template>
						<template #edit-files>
							<FileIcon aria-hidden="true" />
							Edit files
						</template>
						<template #edit-details>
							<InfoIcon aria-hidden="true" />
							Edit details
						</template>
						<template #edit-metadata>
							<BoxIcon aria-hidden="true" />
							Edit metadata
						</template>
						<template #delete>
							<TrashIcon aria-hidden="true" />
							Delete
						</template>
						<template #view-c2pa-info>
							<ScanEyeIcon aria-hidden="true" />
							View C2PA Info
						</template>
						<template #copy-id>
							<ClipboardCopyIcon aria-hidden="true" />
							Copy ID
						</template>
						<template #copy-maven>
							<ClipboardCopyIcon aria-hidden="true" />
							Copy Maven coordinates
						</template>
					</TeleportOverflowMenu>
				</template>
			</ProjectPageVersions>
			<template v-else>
				<p class="ml-2">
					No versions in project. Visit
					<NuxtLink to="settings/versions">
						<span class="font-medium text-green hover:underline">project settings</span> to
					</NuxtLink>
					upload your first version.
				</p>
			</template>
		</template>
	</section>
</template>

<script setup>
import {
	BoxIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	EditIcon,
	ExternalIcon,
	FileIcon,
	InfoIcon,
	LinkIcon,
	MoreVerticalIcon,
	ReportIcon,
	ScanEyeIcon,
	ShareIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import {
	ButtonLink,
	ConfirmModal,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	ProjectPageVersions,
	TeleportOverflowMenu,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { onMounted, useTemplateRef, watch } from 'vue'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import ProjectC2paScanModal from '~/components/ui/moderation/ProjectC2paScanModal.vue'
import { getSignInRouteObj } from '~/composables/auth.ts'
import { reportVersion } from '~/utils/report-helpers.ts'

const route = useRoute()

const { createProjectDownloadUrl, updateVersionsFilterContext } = useCdnDownloadContext()

const tags = useGeneratedState()
const flags = useFeatureFlags()
const modSettings = useModerationSettings()
const auth = await useAuth()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const {
	projectV2: project,
	currentMember,
	invalidate,
	versions,
	versionsLoading,
	versionsLoaded,
	loadVersions,
	cdnDownloadReason,
} = injectProjectPageContext()

const showVersionsLoadingState = computed(
	() =>
		!versions.value?.length &&
		(versionsLoading.value ||
			(!versionsLoaded.value && (project.value?.versions?.length ?? 0) > 0)),
)

// Load versions on mount (client-side)
onMounted(() => {
	loadVersions()
})

const deleteVersionModal = ref()
const selectedVersion = ref(null)
const createProjectVersionModal = useTemplateRef('create-project-version-modal')
const projectC2paScanModal = useTemplateRef('project-c2pa-scan-modal')

const handleOpenCreateVersionModal = () => {
	if (!currentMember.value) return
	createProjectVersionModal.value?.openCreateVersionModal()
}

const handleOpenEditVersionModal = (versionId, projectId, stageId) => {
	if (!currentMember.value) return
	createProjectVersionModal.value?.openEditVersionModal(versionId, projectId, stageId)
}

const emit = defineEmits(['onDownload', 'deleteVersion'])

const baseDropdownId = useId()

function getPrimaryFile(version) {
	return version.files?.find((x) => x.primary) || version.files?.[0]
}

watch(
	() => [route.query.g, route.query.l],
	() => {
		updateVersionsFilterContext(
			queryAsStringArray(route.query.g),
			queryAsStringArray(route.query.l),
		)
	},
	{ immediate: true },
)

function createDownloadUrl(version) {
	const file = getPrimaryFile(version)
	if (!file?.url) return undefined

	return createProjectDownloadUrl(file.url, {
		reason: cdnDownloadReason.value,
	})
}

async function copyToClipboard(text) {
	await navigator.clipboard.writeText(text)
}

async function deleteVersion() {
	const id = selectedVersion.value
	if (!id) return

	startLoading()

	try {
		await client.labrinth.versions_v3.deleteVersion(id)

		addNotification({
			title: 'Version deleted',
			text: 'The version has been successfully deleted.',
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}

	await invalidate()
	selectedVersion.value = null

	stopLoading()
}
</script>
