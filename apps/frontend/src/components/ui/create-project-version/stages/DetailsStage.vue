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
				{{ formatMessage(messages.versionType) }} <span class="text-red">*</span>
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
				{{ formatMessage(messages.versionNumber) }} <span class="text-red">*</span>
			</span>
			<StyledInput
				id="version-number"
				v-model="draftVersion.version_number"
				:disabled="isUploading"
				:placeholder="formatMessage(messages.versionNumberPlaceholder)"
				autocomplete="off"
				:maxlength="32"
			/>
			<span>{{ formatMessage(messages.versionNumberDescription) }}</span>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.versionSubtitle) }}</span>
			<StyledInput
				id="version-number"
				v-model="draftVersion.name"
				:placeholder="formatMessage(messages.versionSubtitlePlaceholder)"
				autocomplete="off"
				:maxlength="256"
				:disabled="isUploading"
			/>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.versionChangelog) }}</span>

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
import {
	Chips,
	defineMessages,
	MarkdownEditor,
	StyledInput,
	Tabs,
	type TabsTab,
	useVIntl,
} from '@modrinth/ui'

import { useImageUpload } from '~/composables/image-upload.ts'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { draftVersion, isUploading, editingVersion, modal } = injectManageVersionContext()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	versionType: {
		id: 'create-project-version.create-modal.stage.details.version-type',
		defaultMessage: 'Version type',
	},
	versionNumber: {
		id: 'create-project-version.create-modal.stage.details.version-number',
		defaultMessage: 'Version number',
	},
	versionNumberPlaceholder: {
		id: 'create-project-version.create-modal.stage.details.version-number-placeholder',
		defaultMessage: 'Enter version number, e.g. 1.2.3-alpha.1',
	},
	versionNumberDescription: {
		id: 'create-project-version.create-modal.stage.details.version-number-description',
		defaultMessage: 'The version number differentiates this specific version from others.',
	},
	versionSubtitle: {
		id: 'create-project-version.create-modal.stage.details.version-subtitle',
		defaultMessage: 'Version subtitle',
	},
	versionSubtitlePlaceholder: {
		id: 'create-project-version.create-modal.stage.details.version-subtitle-placeholder',
		defaultMessage: 'Enter subtitle...',
	},
	versionChangelog: {
		id: 'create-project-version.create-modal.stage.details.version-changelog',
		defaultMessage: 'Version changelog',
	},
	metadataTab: {
		id: 'create-project-version.create-modal.stage.details.metadata-tab',
		defaultMessage: 'Metadata',
	},
	detailsTab: {
		id: 'create-project-version.create-modal.stage.details.details-tab',
		defaultMessage: 'Details',
	},
	filesTab: {
		id: 'create-project-version.create-modal.stage.details.files-tab',
		defaultMessage: 'Files',
	},
})

const editTabs = computed<TabsTab[]>(() => [
	{ label: formatMessage(messages.metadataTab), value: 'metadata' },
	{ label: formatMessage(messages.detailsTab), value: 'add-details' },
	{ label: formatMessage(messages.filesTab), value: 'add-files' },
])

function setEditTab(tab: TabsTab) {
	modal.value?.setStage(tab.value)
}

async function onImageUpload(file: File) {
	const response = await useImageUpload(file, { context: 'version' })
	return response.url
}
</script>
