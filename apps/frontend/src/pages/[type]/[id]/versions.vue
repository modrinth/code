<template>
	<section class="experimental-styles-within overflow-visible">
		<CreateProjectVersionModal
			v-if="currentMember"
			ref="create-project-version-modal"
		></CreateProjectVersionModal>

		<ConfirmModal
			v-if="currentMember"
			ref="deleteVersionModal"
			title="Are you sure you want to delete this version?"
			description="This will remove this version forever (like really forever)."
			:has-to-type="false"
			proceed-label="Delete"
			@proceed="deleteVersion()"
		/>

		<Admonition v-if="!hideVersionsAdmonition && currentMember" type="info" class="mb-4">
			Creating and editing project versions can now be done directly from the
			<NuxtLink to="settings/versions" class="font-medium text-blue hover:underline"
				>project settings</NuxtLink
			>.
			<template #actions>
				<div class="flex gap-2">
					<ButtonStyled color="blue">
						<button
							aria-label="Project Settings"
							class="!shadow-none"
							@click="() => router.push('settings/versions')"
						>
							<SettingsIcon />
							Edit versions
						</button>
					</ButtonStyled>
					<ButtonStyled type="transparent">
						<button
							aria-label="Dismiss"
							class="!shadow-none"
							@click="() => (hideVersionsAdmonition = true)"
						>
							Dismiss
						</button>
					</ButtonStyled>
				</div>
			</template>
		</Admonition>

		<ProjectPageVersions
			v-if="versions.length"
			:project="project"
			:versions="versions"
			:show-files="flags.showVersionFilesInTable"
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
				<ButtonStyled circular type="transparent">
					<a
						v-tooltip="`Download`"
						:href="getPrimaryFile(version).url"
						class="hover:!bg-button-bg [&>svg]:!text-green"
						aria-label="Download"
						@click="emit('onDownload')"
					>
						<DownloadIcon aria-hidden="true" />
					</a>
				</ButtonStyled>
				<ButtonStyled v-if="currentMember" circular type="transparent">
					<OverflowMenu
						v-tooltip="'Edit version'"
						class="hover:!bg-button-bg"
						:dropdown-id="`${baseDropdownId}-edit-${version.id}`"
						:options="[
							{
								id: 'edit-details',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'add-details'),
							},
							{
								id: 'edit-changelog',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'add-changelog'),
							},
							{
								id: 'edit-dependencies',
								action: () =>
									handleOpenEditVersionModal(version.id, project.id, 'add-dependencies'),
								shown: project.project_type !== 'modpack',
							},
							{
								id: 'edit-files',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'add-files'),
							},
						]"
						aria-label="Edit version"
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
						<template #edit-dependencies>
							<BoxIcon aria-hidden="true" />
							Edit dependencies
						</template>
						<template #edit-changelog>
							<AlignLeftIcon aria-hidden="true" />
							Edit changelog
						</template>
					</OverflowMenu>
				</ButtonStyled>
				<ButtonStyled circular type="transparent">
					<OverflowMenu
						v-tooltip="'More options'"
						class="hover:!bg-button-bg"
						:dropdown-id="`${baseDropdownId}-${version.id}`"
						:options="[
							{
								id: 'download',
								color: 'primary',
								hoverFilled: true,
								link: getPrimaryFile(version).url,
								action: () => {
									emit('onDownload')
								},
							},
							{
								id: 'new-tab',
								action: () => {},
								link: `/${project.project_type}/${
									project.slug ? project.slug : project.id
								}/version/${encodeURI(version.displayUrlEnding)}`,
								external: true,
							},
							{
								id: 'copy-link',
								action: () =>
									copyToClipboard(
										`https://modrinth.com/${project.project_type}/${
											project.slug ? project.slug : project.id
										}/version/${encodeURI(version.displayUrlEnding)}`,
									),
							},
							{
								id: 'share',
								action: () => {},
								shown: false,
							},
							{
								id: 'report',
								color: 'red',
								hoverFilled: true,
								action: () => (auth.user ? reportVersion(version.id) : navigateTo('/auth/sign-in')),
								shown: !currentMember,
							},
							{ divider: true, shown: currentMember || flags.developerMode },
							{
								id: 'copy-id',
								action: () => {
									copyToClipboard(version.id)
								},
								shown: currentMember || flags.developerMode,
							},
							{
								id: 'copy-maven',
								action: () => {
									copyToClipboard(`maven.modrinth:${project.slug}:${version.id}`)
								},
								shown: flags.developerMode,
							},
							{ divider: true, shown: !!currentMember },
							{
								id: 'edit-details',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'add-details'),
								shown: !!currentMember,
							},
							{
								id: 'edit-changelog',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'add-changelog'),
								shown: !!currentMember,
							},
							{
								id: 'edit-dependencies',
								action: () =>
									handleOpenEditVersionModal(version.id, project.id, 'add-dependencies'),
								shown: !!currentMember && project.project_type !== 'modpack',
							},
							{
								id: 'edit-files',
								action: () => handleOpenEditVersionModal(version.id, project.id, 'add-files'),
								shown: !!currentMember,
							},
							{
								id: 'delete',
								color: 'red',
								hoverFilled: true,
								action: () => {
									selectedVersion = version.id
									deleteVersionModal?.show()
								},
								shown: !!currentMember,
							},
						]"
						aria-label="More options"
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
						<template #edit-dependencies>
							<BoxIcon aria-hidden="true" />
							Edit dependencies
						</template>
						<template #edit-changelog>
							<AlignLeftIcon aria-hidden="true" />
							Edit changelog
						</template>
						<template #delete>
							<TrashIcon aria-hidden="true" />
							Delete
						</template>
						<template #copy-id>
							<ClipboardCopyIcon aria-hidden="true" />
							Copy ID
						</template>
						<template #copy-maven>
							<ClipboardCopyIcon aria-hidden="true" />
							Copy Maven coordinates
						</template>
					</OverflowMenu>
				</ButtonStyled>
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
	</section>
</template>

<script setup>
import {
	AlignLeftIcon,
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
	SettingsIcon,
	ShareIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	ConfirmModal,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	OverflowMenu,
	ProjectPageVersions,
} from '@modrinth/ui'
import { useLocalStorage } from '@vueuse/core'
import { useTemplateRef } from 'vue'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import { reportVersion } from '~/utils/report-helpers.ts'

const props = defineProps({
	project: {
		type: Object,
		default() {
			return {}
		},
	},
	versions: {
		type: Array,
		default() {
			return []
		},
	},
	currentMember: {
		type: Object,
		default() {
			return null
		},
	},
})

const tags = useGeneratedState()
const flags = useFeatureFlags()
const auth = await useAuth()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { refreshVersions } = injectProjectPageContext()

const deleteVersionModal = ref()
const selectedVersion = ref(null)
const createProjectVersionModal = useTemplateRef('create-project-version-modal')

const handleOpenCreateVersionModal = () => {
	if (!props.currentMember) return
	createProjectVersionModal.value?.openCreateVersionModal()
}

const handleOpenEditVersionModal = (versionId, projectId, stageId) => {
	if (!props.currentMember) return
	createProjectVersionModal.value?.openEditVersionModal(versionId, projectId, stageId)
}

const hideVersionsAdmonition = useLocalStorage(
	'hideVersionsHasMovedAdmonition',
	!props.versions.length,
)

const emit = defineEmits(['onDownload', 'deleteVersion'])

const router = useNativeRouter()

const baseDropdownId = useId()

function getPrimaryFile(version) {
	return version.files.find((x) => x.primary) || version.files[0]
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

	refreshVersions()
	selectedVersion.value = null

	stopLoading()
}
</script>
