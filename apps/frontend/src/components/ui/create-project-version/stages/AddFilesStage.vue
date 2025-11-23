<template>
	<div class="flex flex-col gap-4">
		<FileInput
			:max-size="524288000"
			:accept="acceptFileFromProjectType(project.project_type)"
			prompt="Upload a version"
			class="btn btn-primary"
			aria-label="Upload a version"
			@change="handleFiles"
		>
			<UploadIcon aria-hidden="true" />
		</FileInput>
	</div>
</template>

<script setup lang="ts">
import { UploadIcon } from '@modrinth/assets'
import { FileInput } from '@modrinth/ui'
import { acceptFileFromProjectType } from '@modrinth/utils'

import { injectVersionsContext } from '~/providers/versions'

const router = useNativeRouter()

const { project } = injectVersionsContext()

async function handleFiles(files: any) {
	await router.push({
		name: 'type-id-version-version',
		params: {
			type: project.project_type,
			id: project.slug ? project.slug : project.id,
			version: 'create',
		},
		state: {
			newPrimaryFile: files[0],
		},
	})
}
</script>
