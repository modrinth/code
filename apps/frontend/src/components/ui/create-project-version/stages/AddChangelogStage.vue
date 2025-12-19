<template>
	<div class="w-full">
		<MarkdownEditor
			v-model="draftVersion.changelog"
			:on-image-upload="onImageUpload"
			:max-height="500"
		/>
	</div>
</template>

<script lang="ts" setup>
import { MarkdownEditor } from '@modrinth/ui'

import { useImageUpload } from '~/composables/image-upload.ts'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { draftVersion } = injectManageVersionContext()

async function onImageUpload(file: File) {
	const response = await useImageUpload(file, { context: 'version' })
	return response.url
}
</script>
