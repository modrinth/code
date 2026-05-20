<template>
	<Tabs
		v-if="editingVersion"
		value="add-details"
		:tabs="editTabs"
		class="mb-5 border border-solid border-surface-5 !shadow-none !drop-shadow-none"
		@change="setEditTab"
	/>
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
				hide-checkmark-icon
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
import { Chips, MarkdownEditor, StyledInput, Tabs, type TabsTab } from '@modrinth/ui'

import { useImageUpload } from '~/composables/image-upload.ts'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { draftVersion, isUploading, editingVersion, modal } = injectManageVersionContext()

const editTabs: TabsTab[] = [
	{ label: 'Metadata', value: 'metadata' },
	{ label: 'Details', value: 'add-details' },
	{ label: 'Files', value: 'add-files' },
]

function setEditTab(tab: TabsTab) {
	modal.value?.setStage(tab.value)
}

async function onImageUpload(file: File) {
	const response = await useImageUpload(file, { context: 'version' })
	return response.url
}
</script>
