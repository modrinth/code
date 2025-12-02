<template>
	<div class="mb-4 flex w-[576px] max-w-[576px] flex-col gap-4">
		<FileInput
			prompt="Drag and drop to upload or click to select"
			aria-label="Upload additional file"
			multiple
			long-style
			:accept="acceptFileFromProjectType(project.project_type)"
			:max-size="524288000"
			@change="handleNewFiles"
		>
			<UploadIcon aria-hidden="true" />
		</FileInput>

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
import { UploadIcon } from '@modrinth/assets'
import { FileInput } from '@modrinth/ui'
import Admonition from '@modrinth/ui/src/components/base/Admonition.vue'
import { acceptFileFromProjectType } from '@modrinth/utils'

import { useManageVersion } from '~/composables/versions/manage-version'
import { injectVersionsContext } from '~/providers/versions'

import VersionFileRow from '../components/VersionFileRow.vue'
const { project } = injectVersionsContext()
const { formatMessage } = useVIntl()

const { draftVersion, setPrimaryFile } = useManageVersion()

function handleNewFiles(newFiles: File[]) {
	newFiles.forEach((file) => draftVersion.value.files.push(file))
}

function onRemoveFile(index: number) {
	draftVersion.value.files.splice(index, 1)
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
