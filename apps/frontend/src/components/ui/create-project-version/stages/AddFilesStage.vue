<template>
	<div class="mb-4 flex w-full flex-col gap-4">
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

		<template v-if="filesToAdd.length">
			<div class="flex flex-col gap-2">
				<span class="text-base font-semibold text-contrast">Uploaded files</span>
				<div class="flex flex-col gap-2.5">
					<VersionFileRow
						v-for="(versionFile, idx) in filesToAdd"
						:key="versionFile.file.name"
						:file="versionFile.file"
						:is-primary="idx === 0"
						@set-primary-file="handleSetPrimaryFile(idx)"
						@remove="handleRemoveFile(idx)"
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

import VersionFileRow from '../components/VersionFileRow.vue'

const { projectV2 } = injectProjectPageContext()
const { formatMessage } = useVIntl()

const { draftVersion, filesToAdd, existingFilesToDelete, setPrimaryFile, setInferredVersionData } =
	useManageVersion()

const addDetectedData = async () => {
	const primaryFile = filesToAdd.value[0]?.file
	if (!primaryFile) return

	try {
		const inferredData = await setInferredVersionData(primaryFile, projectV2.value)
		const mappedInferredData = {
			...inferredData,
			version_title: inferredData.name || '',
		}

		draftVersion.value = {
			...draftVersion.value,
			...mappedInferredData,
		}
	} catch (err) {
		console.error('Error parsing version file data', err)
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
