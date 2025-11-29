<template>
	<MarkdownEditor
		v-model="changelog"
		:on-image-upload="onImageUpload"
		:style="'background-color: red;'"
		:max-height="500"
	/>
</template>

<script lang="ts" setup>
import MarkdownEditor from '@modrinth/ui/src/components/base/MarkdownEditor.vue'

import { useImageUpload } from '~/composables/image-upload.ts'

const changelog = ref<string>('')

async function onImageUpload(file: File) {
	const response = await useImageUpload(file, { context: 'version' })
	return response.url
}
</script>
