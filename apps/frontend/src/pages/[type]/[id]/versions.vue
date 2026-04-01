<template>
	<section class="experimental-styles-within overflow-visible">
		<!-- Loading state -->
		<div
			v-if="versionsLoading && !versions?.length"
			class="flex items-center justify-center gap-2 py-8"
		>
			<SpinnerIcon class="animate-spin" />
			<span>Loading versions...</span>
		</div>

		<template v-else>
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

			<ProjectPageVersions
				v-if="versions?.length"
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
					<ButtonStyled color="brand" size="small">
						<a
							:href="getPrimaryFile(version).url"
							aria-label="Download"
							@click="emit('onDownload')"
						>
							Download
						</a>
					</ButtonStyled>
					<ButtonStyled v-if="currentMember" size="small">
						<OverflowMenu
							:dropdown-id="`${baseDropdownId}-edit-${version.id}`"
							:options="[
								{
									id: 'edit-metadata',
									action: () => handleOpenEditVersionModal(version.id, project.id, 'metadata'),
								},
								{
									id: 'edit-details',
									action: () => handleOpenEditVersionModal(version.id, project.id, 'add-details'),
								},
								{
									id: 'edit-files',
									action: () => handleOpenEditVersionModal(version.id, project.id, 'add-files'),
								},
							]"
							aria-label="Edit version"
						>
							Edit
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
		</template>
	</section>
</template>

<script setup>
import { BoxIcon, FileIcon, InfoIcon, SpinnerIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmModal,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	OverflowMenu,
	ProjectPageVersions,
} from '@modrinth/ui'
import { onMounted, useTemplateRef } from 'vue'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'

const tags = useGeneratedState()
const flags = useFeatureFlags()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const {
	projectV2: project,
	currentMember,
	invalidate,
	versions,
	versionsLoading,
	loadVersions,
} = injectProjectPageContext()

// Load versions on mount (client-side)
onMounted(() => {
	loadVersions()
})

const deleteVersionModal = ref()
const selectedVersion = ref(null)
const createProjectVersionModal = useTemplateRef('create-project-version-modal')

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
	return version.files.find((x) => x.primary) || version.files[0]
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
