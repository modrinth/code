<template>
	<MarkdownEditor
		v-model="draftVersion.changelog"
		:on-image-upload="onImageUpload"
		:max-height="500"
	/>
</template>

<script lang="ts" setup>
import { MarkdownEditor } from '@modrinth/ui'

import { useImageUpload } from '~/composables/image-upload.ts'
import { useManageVersion } from '~/composables/versions/manage-version'

const { draftVersion } = useManageVersion()

async function onImageUpload(file: File) {
	const response = await useImageUpload(file, { context: 'version' })
	return response.url
}
</script>
