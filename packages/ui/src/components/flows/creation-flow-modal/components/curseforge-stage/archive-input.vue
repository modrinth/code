<template>
	<div
		v-if="file"
		class="flex h-10 min-w-0 items-center gap-2 rounded-[14px] border border-solid border-surface-5 bg-surface-4 px-4"
	>
		<FileIcon class="size-5 shrink-0 text-primary" aria-hidden="true" />
		<span class="min-w-0 flex-1 truncate font-semibold text-contrast" :title="file.name">
			{{ file.name }}
		</span>
		<IconButton
			type="quiet"
			size="sm"
			:label="formatMessage(messages.removeFile, { filename: file.name })"
			@click="file = null"
		>
			<XIcon aria-hidden="true" />
		</IconButton>
	</div>
	<DropzoneFileInput
		v-else
		size="small"
		accept=".zip"
		:multiple="false"
		:primary-prompt="null"
		:secondary-prompt="formatMessage(messages.uploadPrompt)"
		class="!rounded-[20px] !border-[1.5px] !border-surface-4 !bg-surface-2 focus-within:outline focus-within:outline-2 focus-within:outline-brand"
		@change="file = $event[0] ?? null"
	/>
</template>

<script setup lang="ts">
import { FileIcon, XIcon } from '@modrinth/assets'

import { IconButton } from '#ui/components/base/buttons'
import DropzoneFileInput from '#ui/components/base/DropzoneFileInput.vue'
import { useVIntl } from '#ui/composables/i18n'

import { curseforgeMessages as messages } from '../../curseforge'

const file = defineModel<File | null>({ required: true })
const { formatMessage } = useVIntl()
</script>
