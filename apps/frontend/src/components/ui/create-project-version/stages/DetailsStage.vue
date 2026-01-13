<template>
	<div class="flex w-full flex-col gap-6">
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version type <span class="text-red">*</span>
			</span>
			<Chips
				v-model="draftVersion.version_type"
				:items="['release', 'beta', 'alpha']"
				:never-empty="true"
				:capitalize="true"
				:disabled="isUploading"
			/>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version number <span class="text-red">*</span>
			</span>
			<input
				id="version-number"
				v-model="draftVersion.version_number"
				:disabled="isUploading"
				placeholder="Enter version number, e.g. 1.2.3-alpha.1"
				type="text"
				autocomplete="off"
				maxlength="32"
			/>
			<span> The version number differentiates this specific version from others. </span>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast"> Version subtitle </span>
			<input
				id="version-number"
				v-model="draftVersion.name"
				placeholder="Enter subtitle..."
				type="text"
				autocomplete="off"
				maxlength="256"
				:disabled="isUploading"
			/>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast"> Version changlog </span>

			<div class="w-full">
				<MarkdownEditor
					v-model="draftVersion.changelog"
					:on-image-upload="onImageUpload"
					:min-height="150"
					:disabled="isUploading"
				/>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import { Chips, MarkdownEditor } from '@modrinth/ui'

import { useImageUpload } from '~/composables/image-upload.ts'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { draftVersion, isUploading } = injectManageVersionContext()

async function onImageUpload(file: File) {
	const response = await useImageUpload(file, { context: 'version' })
	return response.url
}
</script>
