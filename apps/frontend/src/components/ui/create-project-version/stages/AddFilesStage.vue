<template>
	<div class="flex w-full flex-col gap-4">
		<DropzoneFileInput
			aria-label="Upload additional file"
			multiple
			long-style
			:accept="acceptFileFromProjectType(projectV2.project_type)"
			:max-size="524288000"
			@change="handleNewFiles"
		/>

		<Admonition v-if="hasSupplementaryFiles" type="warning">
			{{ formatMessage(messages.addFilesAdmonition) }}
		</Admonition>

		<template v-if="filesToAdd.length || draftVersion.existing_files?.length">
			<div class="flex flex-col gap-2">
				<span class="text-base font-semibold text-contrast">Uploaded files</span>
				<div class="flex flex-col gap-2.5">
					<VersionFileRow
						v-for="versionFile in draftVersion.existing_files"
						:key="versionFile.filename"
						:name="versionFile.filename"
						:is-primary="versionFile.primary"
						:initialFileType="versionFile.file_type"
						:editingVersion="editingVersion"
						@set-file-type="(type) => (versionFile.file_type = type)"
						:onRemove="
							versionFile.primary
								? undefined
								: () => handleRemoveExistingFile(versionFile.hashes.sha1 || '')
						"
					/>
					<VersionFileRow
						v-for="(versionFile, idx) in filesToAdd"
						:key="versionFile.file.name"
						:name="versionFile.file.name"
						:is-primary="idx === 0 && !draftVersion.existing_files?.some((f) => f.primary)"
						:initialFileType="versionFile.fileType"
						:editingVersion="editingVersion"
						@set-primary-file="handleSetPrimaryFile(idx)"
						@set-file-type="(type) => (versionFile.fileType = type)"
						:onRemove="() => handleRemoveFile(idx)"
					/>
				</div>
				<span>
					The primary file is the default file a user downloads when installing the project.
				</span>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import { Admonition, DropzoneFileInput, injectProjectPageContext } from '@modrinth/ui'
import { acceptFileFromProjectType } from '@modrinth/utils'

import { useManageVersion } from '~/composables/versions/manage-version'

import type { Labrinth } from '@modrinth/api-client'
import VersionFileRow from '../components/VersionFileRow.vue'

const { projectV2 } = injectProjectPageContext()
const { formatMessage } = useVIntl()

const {
	draftVersion,
	filesToAdd,
	existingFilesToDelete,
	setPrimaryFile,
	setInferredVersionData,
	editingVersion,
	projectType,
} = useManageVersion()

const addDetectedData = async () => {
	if (editingVersion.value) return

	const primaryFile = filesToAdd.value[0]?.file
	if (!primaryFile) return

	try {
		const inferredData = await setInferredVersionData(primaryFile, projectV2.value)
		const mappedInferredData: Partial<Labrinth.Versions.v3.DraftVersion> = {
			...inferredData,
			name: inferredData.name || '',
		}

		draftVersion.value = {
			...draftVersion.value,
			...mappedInferredData,
		}
	} catch (err) {
		console.error('Error parsing version file data', err)
	}

	if (projectType.value === 'resourcepack') {
		draftVersion.value.loaders = ['minecraft']
	}
}

function handleNewFiles(newFiles: File[]) {
	newFiles.forEach((file) => filesToAdd.value.push({ file }))
	addDetectedData()
}

function handleRemoveFile(index: number) {
	filesToAdd.value.splice(index, 1)
	addDetectedData()
}

function handleRemoveExistingFile(sha1: string) {
	existingFilesToDelete.value.push(sha1)
	draftVersion.value.existing_files = draftVersion.value.existing_files?.filter(
		(file) => file.hashes.sha1 !== sha1,
	)
}

function handleSetPrimaryFile(index: number) {
	setPrimaryFile(index)
	addDetectedData()
}

const hasSupplementaryFiles = computed(() => filesToAdd.value.length > 1)

const messages = defineMessages({
	addFilesAdmonition: {
		id: 'create-project-version.create-modal.stage.add-files.admonition',
		defaultMessage:
			'Supplementary files are for supporting resources like source code, not for alternative versions or variants.',
	},
})
</script>
