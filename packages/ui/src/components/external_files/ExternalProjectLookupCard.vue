<script setup lang="ts">
import {
	ClipboardCopyIcon,
	CurseForgeIcon,
	FileIcon,
	LinkIcon,
	UnknownIcon,
} from '@modrinth/assets'
import { Menu } from 'floating-vue'
import { computed } from 'vue'

import { ButtonStyled, CopyCode, TagItem } from '#ui/components'
import { commonMessages } from '#ui/utils/common-messages'

import { defineMessages, useVIntl } from '../../composables/i18n'
import type { ExternalLicenseStatus } from './types.ts'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	curseforgeProject: {
		id: 'external-files.lookup-card.curseforge-project',
		defaultMessage: 'CurseForge project',
	},
	projectLink: {
		id: 'external-files.lookup-card.project-link',
		defaultMessage: 'Project link',
	},
	allowedLabel: {
		id: 'external-files.lookup-card.allowed-label',
		defaultMessage: 'Allowed:',
	},
	proofLabel: {
		id: 'external-files.lookup-card.proof-label',
		defaultMessage: 'Proof:',
	},
	notesLabel: {
		id: 'external-files.lookup-card.notes-label',
		defaultMessage: 'Notes:',
	},
	filesHeading: {
		id: 'external-files.lookup-card.files-heading',
		defaultMessage: 'Files',
	},
	noFiles: {
		id: 'external-files.lookup-card.no-files',
		defaultMessage: 'No files available for external project.',
	},
	firstRecognizedFileName: {
		id: 'external-files.lookup-card.first-identified-name',
		defaultMessage: 'First identified name:',
	},
	unknownFileName: {
		id: 'external-files.lookup-card.unknown-file-name',
		defaultMessage: 'Unknown',
	},
	sha1Label: {
		id: 'external-files.lookup-card.sha1-label',
		defaultMessage: 'SHA-1:',
	},
	lastEdited: {
		id: 'external-files.lookup-card.last-edited',
		defaultMessage: 'Last edited on {date} by {user}',
	},
})

const statusMessages = defineMessages({
	yes: {
		id: 'external-files.lookup-card.status.yes',
		defaultMessage: 'Yes',
	},
	'with-attribution-and-source': {
		id: 'external-files.lookup-card.status.with-attribution-and-source',
		defaultMessage: 'With attribution and source',
	},
	'with-attribution': {
		id: 'external-files.lookup-card.status.with-attribution',
		defaultMessage: 'With attribution',
	},
	no: {
		id: 'external-files.lookup-card.status.no',
		defaultMessage: 'No',
	},
	'permanent-no': {
		id: 'external-files.lookup-card.status.permanent-no',
		defaultMessage: 'Permanent no',
	},
	unidentified: {
		id: 'external-files.lookup-card.status.unidentified',
		defaultMessage: 'Unidentified',
	},
})

const props = defineProps<{
	title: string | null
	link: string | null
	state: ExternalLicenseStatus
	proof: string | null
	notes: string | null
	last_updated?: string
	last_updated_by?: string
	files: {
		sha1: string
		name: string | null
	}[]
}>()

const stateLabel = computed(() => formatMessage(statusMessages[props.state]))

const lastEditedText = computed(() =>
	props.last_updated
		? formatMessage(messages.lastEdited, {
				date: props.last_updated,
				user: props.last_updated_by ?? '',
			})
		: '',
)

async function copyProjectLink() {
	if (!props.link) return
	await navigator.clipboard.writeText(props.link)
}
</script>

<template>
	<div class="bg-surface-3 p-4 rounded-2xl flex flex-col gap-3">
		<span class="text-contrast font-semibold">{{ title }}</span>
		<div v-if="!!link" class="flex gap-2 items-center">
			<a
				class="flex items-center gap-2 font-medium hover:underline focus:underline w-fit text-blue"
				:href="link"
				target="_blank"
			>
				<template v-if="link?.includes('curseforge.com')">
					<CurseForgeIcon class="size-5 shrink-0" /> {{ formatMessage(messages.curseforgeProject) }}
				</template>
				<template v-else>
					<LinkIcon class="size-5 shrink-0" /> {{ formatMessage(messages.projectLink) }}
				</template>
			</a>
			<ButtonStyled circular type="transparent" size="small">
				<button v-tooltip="formatMessage(commonMessages.copyLinkButton)" @click="copyProjectLink">
					<ClipboardCopyIcon />
				</button>
			</ButtonStyled>
		</div>

		<div class="grid grid-cols-[auto_1fr] items-center gap-x-4 gap-y-2">
			<div class="font-medium">{{ formatMessage(messages.allowedLabel) }}</div>
			<div>
				<TagItem>
					{{ stateLabel }}
				</TagItem>
			</div>
			<div class="font-medium">{{ formatMessage(messages.proofLabel) }}</div>
			<div>{{ proof }}</div>
			<div class="font-medium">{{ formatMessage(messages.notesLabel) }}</div>
			<div>{{ notes }}</div>
		</div>
		<div class="bg-surface-2 p-4 rounded-2xl flex flex-col gap-3">
			<span class="text-contrast font-semibold">{{ formatMessage(messages.filesHeading) }}</span>
			<span v-if="!(files?.length > 0)" class="text-secondary">
				{{ formatMessage(messages.noFiles) }}
			</span>
			<div v-else class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-2">
				<Menu v-for="file in files" :key="file.sha1" :delay="{ hide: 50, show: 0 }">
					<div
						class="line-clamp-1 truncate px-2 py-1 flex gap-2 rounded-xl items-center border-solid border-2 border-surface-5 text-sm font-medium text-secondary"
					>
						<FileIcon class="shrink-0 size-4" /> {{ file.name ?? file.sha1 }}
					</div>
					<template #popper>
						<div class="text-primary p-2 grid grid-cols-[auto_1fr] gap-x-4 gap-y-2">
							<div>{{ formatMessage(messages.firstRecognizedFileName) }}</div>
							<div v-if="file.name" class="text-sm">
								{{ file.name }}
							</div>
							<div v-else class="text-secondary flex items-center gap-2 text-sm">
								<UnknownIcon /> {{ formatMessage(messages.unknownFileName) }}
							</div>
							<div>{{ formatMessage(messages.sha1Label) }}</div>
							<div class="text-sm"><CopyCode :text="file.sha1" /></div>
						</div>
					</template>
				</Menu>
			</div>
		</div>
		<div v-if="last_updated" class="pt-4 border-t-[1px] border-solid border-surface-5">
			{{ lastEditedText }}
		</div>
	</div>
</template>
