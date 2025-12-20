<template>
	<div class="flex w-full flex-col gap-4 sm:w-[512px]">
		<template v-if="!(filesToAdd.length || draftVersion.existing_files?.length)">
			<DropzoneFileInput
				aria-label="Upload file"
				multiple
				:accept="acceptFileFromProjectType(projectV2.project_type)"
				:max-size="524288000"
				@change="handleNewFiles"
			/>
		</template>

		<template v-else>
			<div class="flex flex-col gap-2">
				<span class="text-base font-semibold text-contrast">Primary file</span>
				<div class="flex flex-col gap-2.5">
					<VersionFileRow
						v-if="primaryFile"
						:key="primaryFile.name"
						:name="primaryFile.name"
						:is-primary="true"
						:editing-version="editingVersion"
						:on-remove="undefined"
						@set-primary-file="
							(file) => {
								if (file && !editingVersion) filesToAdd[0] = { file }
							}
						"
					/>
				</div>
				<span>
					The primary file is the default file a user downloads when installing the project.
				</span>
			</div>

			<div class="flex flex-col gap-2">
				<div class="flex flex-col gap-2">
					<Admonition v-if="hasSupplementaryFiles" type="warning">
						{{ formatMessage(messages.addFilesAdmonition) }}
					</Admonition>

					<span class="text-base font-semibold text-contrast">Supplementary files</span>

					<DropzoneFileInput
						aria-label="Upload additional file"
						multiple
						:accept="acceptFileFromProjectType(projectV2.project_type)"
						:max-size="524288000"
						size="small"
						:primary-prompt="null"
						secondary-prompt="Drag and drop files or click to browse"
						@change="handleNewFiles"
					/>

					<div v-if="hasSupplementaryFiles" class="flex flex-col gap-2.5">
						<VersionFileRow
							v-for="versionFile in supplementaryExistingFiles"
							:key="versionFile.filename"
							:name="versionFile.filename"
							:is-primary="false"
							:initial-file-type="versionFile.file_type"
							:editing-version="editingVersion"
							:on-remove="() => handleRemoveExistingFile(versionFile.hashes.sha1 || '')"
							@set-file-type="(type) => (versionFile.file_type = type)"
						/>
						<VersionFileRow
							v-for="(versionFile, idx) in supplementaryNewFiles"
							:key="versionFile.file.name"
							:name="versionFile.file.name"
							:is-primary="false"
							:initial-file-type="versionFile.fileType"
							:editing-version="editingVersion"
							:on-remove="() => handleRemoveFile(idx + (primaryFile?.existing ? 0 : 1))"
							@set-file-type="(type) => (versionFile.fileType = type)"
							@set-primary-file="handleSetPrimaryFile(idx + (primaryFile?.existing ? 0 : 1))"
						/>
					</div>
				</div>
				<span>
					You can optionally add supplementary files such as source code, documentation, or required
					resource packs.
				</span>
			</div>
		</template>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import { Admonition, DropzoneFileInput, injectProjectPageContext } from '@modrinth/ui'
import { acceptFileFromProjectType } from '@modrinth/utils'

import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

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
} = injectManageVersionContext()

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
}

// add detected data when the primary file changes
watch(
	() => filesToAdd.value[0]?.file,
	() => addDetectedData(),
)

function handleNewFiles(newFiles: File[]) {
	// detect primary file if no primary file is set
	const primaryFileIndex = primaryFile.value ? null : detectPrimaryFileIndex(newFiles)

	newFiles.forEach((file) => filesToAdd.value.push({ file }))

	if (primaryFileIndex !== null) {
		if (primaryFileIndex) setPrimaryFile(primaryFileIndex)
	}
}

function handleRemoveFile(index: number) {
	filesToAdd.value.splice(index, 1)
}

function detectPrimaryFileIndex(files: File[]): number {
	const extensionPriority = ['.jar', '.zip', '.litemod', '.mrpack', '.mrpack-primary']

	for (const ext of extensionPriority) {
		const matches = files.filter((file) => file.name.toLowerCase().endsWith(ext))
		if (matches.length > 0) {
			const shortest = matches.reduce((a, b) => (a.name.length < b.name.length ? a : b))
			return files.indexOf(shortest)
		}
	}

	return 0
}

function handleRemoveExistingFile(sha1: string) {
	existingFilesToDelete.value.push(sha1)
	draftVersion.value.existing_files = draftVersion.value.existing_files?.filter(
		(file) => file.hashes.sha1 !== sha1,
	)
}

function handleSetPrimaryFile(index: number) {
	setPrimaryFile(index)
}

interface PrimaryFile {
	name: string
	fileType?: string
	existing?: boolean
}

const primaryFile = computed<PrimaryFile | null>(() => {
	const existingPrimaryFile = draftVersion.value.existing_files?.[0]
	if (existingPrimaryFile) {
		return {
			name: existingPrimaryFile.filename,
			fileType: existingPrimaryFile.file_type,
			existing: true,
		}
	}

	const addedPrimaryFile = filesToAdd.value[0]
	if (addedPrimaryFile) {
		return {
			name: addedPrimaryFile.file.name,
			fileType: addedPrimaryFile.fileType,
			existing: false,
		}
	}

	return null
})

const supplementaryNewFiles = computed(() => {
	if (primaryFile.value?.existing) {
		return filesToAdd.value
	} else {
		return filesToAdd.value.slice(1)
	}
})

const supplementaryExistingFiles = computed(() => {
	if (primaryFile.value?.existing) {
		return draftVersion.value.existing_files?.slice(1)
	} else {
		return draftVersion.value.existing_files
	}
})

const hasSupplementaryFiles = computed(
	() => filesToAdd.value.length + (draftVersion.value.existing_files?.length || 0) > 1,
)

const messages = defineMessages({
	addFilesAdmonition: {
		id: 'create-project-version.create-modal.stage.add-files.admonition',
		defaultMessage:
			'Supplementary files are for supporting resources like source code, not for alternative versions or variants.',
	},
})
</script>
