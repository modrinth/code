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
		v-model:processing="processing"
		size="small"
		:accept="accept"
		:multiple="false"
		:local-files="localFiles"
		:primary-prompt="null"
		:secondary-prompt="formatMessage(messages.uploadPrompt)"
		:processing-prompt="formatMessage(messages.processingFile)"
		class="!rounded-[20px] !border-[1.5px] !border-surface-4 !bg-surface-2 focus-within:outline focus-within:outline-2 focus-within:outline-brand data-[highlighted]:!border-brand data-[highlighted]:!bg-brand-highlight"
		@change="file = $event[0] ?? null"
		@local-change="file = $event[0] ?? null"
	/>
</template>

<script setup lang="ts">
import { FileIcon, XIcon } from '@modrinth/assets'

import { IconButton } from '#ui/components/base/buttons'
import DropzoneFileInput from '#ui/components/base/DropzoneFileInput.vue'
import { useVIntl } from '#ui/composables/i18n'

import type { CurseForgePackFile } from '../../creation-flow-context'
import { curseforgeMessages as messages } from '../../curseforge'

const { accept = '.zip', localFiles = true } = defineProps<{
	accept?: string
	localFiles?: boolean
}>()

const file = defineModel<CurseForgePackFile | null>({ required: true })
const processing = defineModel<boolean>('processing', { default: false })
const { formatMessage } = useVIntl()
</script>
