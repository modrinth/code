<template>
	<div class="flex w-full flex-col gap-6">
		<Chips
			v-if="editingVersion"
			:model-value="'details'"
			:items="editTabs"
			size="small"
			aria-label="Version edit sections"
			hide-checkmark-icon
			@update:model-value="setEditTab"
		/>

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
			<StyledInput
				id="version-number"
				v-model="draftVersion.version_number"
				:disabled="isUploading"
				placeholder="Enter version number, e.g. 1.2.3-alpha.1"
				autocomplete="off"
				:maxlength="32"
			/>
			<span> The version number differentiates this specific version from others. </span>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast"> Version subtitle </span>
			<StyledInput
				id="version-number"
				v-model="draftVersion.name"
				placeholder="Enter subtitle..."
				autocomplete="off"
				:maxlength="256"
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
import { Chips, MarkdownEditor, StyledInput } from '@modrinth/ui'

import { useImageUpload } from '~/composables/image-upload.ts'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { draftVersion, isUploading, editingVersion, modal } = injectManageVersionContext()

type EditVersionTab = 'metadata' | 'details' | 'files'

const editTabs: EditVersionTab[] = ['metadata', 'details', 'files']
const editTabToStage: Record<EditVersionTab, string> = {
	metadata: 'metadata',
	details: 'add-details',
	files: 'add-files',
}

const isEditVersionTab = (tab: string): tab is EditVersionTab =>
	editTabs.some((candidate) => candidate === tab)

function setEditTab(tab: string | null | undefined) {
	if (!tab || !isEditVersionTab(tab)) return
	modal.value?.setStage(editTabToStage[tab])
}

async function onImageUpload(file: File) {
	const response = await useImageUpload(file, { context: 'version' })
	return response.url
}
</script>
