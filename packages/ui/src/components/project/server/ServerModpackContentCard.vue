<template>
	<div class="flex gap-1.5 items-center justify-between px-3 pr-1.5 py-1.5 rounded-[4px] bg-bg">
		<div class="grid grid-cols-[auto_1fr] gap-1.5 items-center">
			<Avatar :src="icon" size="34px" class="!rounded-xl !shadow-none" raised />
			<div class="flex flex-col items-start overflow-hidden">
				<div
					v-tooltip="showCustomModpackTooltip ? formatMessage(messages.customModpackTooltip) : name"
					class="truncate font-semibold text-sm max-w-full"
					:class="onclickName ? 'hover:underline cursor-pointer' : ''"
					@click="onclickName"
				>
					{{ name }}
				</div>
				<div
					v-if="filename"
					v-tooltip="filename"
					class="truncate text-sm text-secondary max-w-full"
				>
					{{ filename }}
				</div>
				<div
					v-if="versionNumber"
					v-tooltip="versionNumber"
					class="truncate font-medium text-sm max-w-full"
					:class="onclickVersion ? 'hover:underline cursor-pointer' : ''"
					@click="onclickVersion"
				>
					{{ versionNumber }}
				</div>
			</div>
		</div>
		<ButtonStyled v-if="onclickDownload" circular type="transparent">
			<button v-tooltip="formatMessage(messages.downloadModpack)" @click="onclickDownload">
				<DownloadIcon />
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { DownloadIcon } from '@modrinth/assets/generated-icons'

import { defineMessages, useVIntl } from '../../../composables'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'

defineProps<{
	name: string
	versionNumber?: string
	filename?: string
	icon?: string
	onclickName?: () => void
	onclickVersion?: () => void
	onclickDownload?: () => void
	showCustomModpackTooltip?: boolean
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	customModpackTooltip: {
		id: `project.server.customModpackTooltip`,
		defaultMessage: 'This project uses a custom modpack',
	},
	downloadModpack: {
		id: `project.about.server.downloadModpack`,
		defaultMessage: 'Download modpack',
	},
})
</script>
