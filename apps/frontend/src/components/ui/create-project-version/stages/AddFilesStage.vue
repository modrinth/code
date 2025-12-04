<template>
	<div class="mb-4 flex w-dvw max-w-[576px] flex-col gap-4">
		<DropzoneFileInput
			aria-label="Upload additional file"
			multiple
			long-style
			:accept="acceptFileFromProjectType(project.project_type)"
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
import { Admonition, DropzoneFileInput } from '@modrinth/ui'
import { acceptFileFromProjectType } from '@modrinth/utils'

import { useManageVersion } from '~/composables/versions/manage-version'
import { inferVersionInfo } from '~/helpers/infer'
import { injectVersionsContext } from '~/providers/versions'

import VersionFileRow from '../components/VersionFileRow.vue'
const { project } = injectVersionsContext()
const { formatMessage } = useVIntl()

const tags = useGeneratedState()

const { draftVersion, setPrimaryFile } = useManageVersion()

// should be in infer.js, but gotta refactor that to ts first
interface InferredVersionInfo {
	name?: string
	version_number?: string
	version_type?: 'alpha' | 'beta' | 'release'
	loaders?: string[]
	game_versions?: string[]
}

const addDetectedData = async () => {
	try {
		const primaryFile = draftVersion.value.files[0]
		const inferredData = (await inferVersionInfo(
			primaryFile,
			project,
			tags.value.gameVersions,
		)) as InferredVersionInfo

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
