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

		<template v-if="draftVersion.files.length">
			<div class="flex flex-col gap-2">
				<span class="text-base font-semibold text-contrast">Uploaded files</span>
				<div class="flex flex-col gap-2.5">
					<VersionFileRow
						v-for="(file, idx) in draftVersion.files"
						:key="file.name"
						:file="file"
						:is-primary="idx === 0"
						@set-primary-file="setPrimaryFile(idx)"
						@remove="onRemoveFile(idx)"
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

const { draftVersion, setPrimaryFile, setInferredVersionData } = useManageVersion()

const addDetectedData = async () => {
	const primaryFile = draftVersion.value.files[0]
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
	newFiles.forEach((file) => draftVersion.value.files.push(file))
	addDetectedData()
}

function onRemoveFile(index: number) {
	draftVersion.value.files.splice(index, 1)
	addDetectedData()
}

const hasSupplementaryFiles = computed(() => draftVersion.value.files.length > 1)

const messages = defineMessages({
	addFilesAdmonition: {
		id: 'create-project-version.create-modal.stage.add-files.admonition',
		defaultMessage:
			'Supplementary files are for supporting resources like source code, not for alternative versions or variants.',
	},
})
</script>
