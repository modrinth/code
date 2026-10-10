<template>
	<TeleportOverflowMenu
		type="quiet"
		:label="formatMessage(messages.editVersionTooltip)"
		:tooltip="formatMessage(messages.editVersionTooltip)"
		class="hover:!bg-button-bg"
		:class="{ '[&>svg]:!text-brand': color === 'brand' }"
		:options="options"
	>
		<EditIcon aria-hidden="true" />
	</TeleportOverflowMenu>
</template>

<script setup lang="ts">
import { BoxIcon, EditIcon, FileIcon, InfoIcon } from '@modrinth/assets'
import { defineMessages, TeleportOverflowMenu, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import type { EditVersionStage } from '~/providers/version/manage-version-modal'

withDefaults(defineProps<{ color?: 'brand' | 'default' }>(), { color: 'brand' })

const emit = defineEmits<{ edit: [stage: EditVersionStage] }>()
const { formatMessage } = useVIntl()
const options = computed(() => [
	{
		id: 'metadata',
		label: formatMessage(messages.editMetadataOption),
		icon: BoxIcon,
		action: () => emit('edit', 'metadata'),
	},
	{
		id: 'add-details',
		label: formatMessage(messages.editDetailsOption),
		icon: InfoIcon,
		action: () => emit('edit', 'add-details'),
	},
	{
		id: 'add-files',
		label: formatMessage(messages.editFilesOption),
		icon: FileIcon,
		action: () => emit('edit', 'add-files'),
	},
])

const messages = defineMessages({
	editVersionTooltip: {
		id: 'project.versions.edit-version-tooltip',
		defaultMessage: 'Edit version',
	},
	editFilesOption: {
		id: 'project.versions.edit-files-option',
		defaultMessage: 'Edit files',
	},
	editDetailsOption: {
		id: 'project.versions.edit-details-option',
		defaultMessage: 'Edit details',
	},
	editMetadataOption: {
		id: 'project.versions.edit-metadata-option',
		defaultMessage: 'Edit metadata',
	},
})
</script>
