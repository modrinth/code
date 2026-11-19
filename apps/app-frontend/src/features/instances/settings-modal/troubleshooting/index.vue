<script setup lang="ts">
import {
	CheckIcon,
	CodeIcon,
	CopyIcon,
	DownloadIcon,
	FileIcon,
	LinkIcon,
	LoaderCircleIcon,
	TextCursorInputIcon,
} from '@modrinth/assets'
import {
	Button,
	type ButtonMenuOption,
	defineMessages,
	injectNotificationManager,
	SplitButton,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import {
	exportDebugInfo,
	isExportingDebugInfo,
} from '@/components/ui/download-manager/debug-info-export'
import { type ContentListFormat, getContentList } from '@/platform/app-lib/instances/commands'
import { useCopyAsPaste } from '@/shared/composables/use-copy-as-paste'

import { injectInstanceSettings } from '../instance-settings-context'

const { formatMessage } = useVIntl()
const copyAsPaste = useCopyAsPaste()
const { addNotification, handleError } = injectNotificationManager()
const { instance } = injectInstanceSettings()

const messages = defineMessages({
	contentListTitle: {
		id: 'instance.settings.tabs.troubleshooting.content-list.title',
		defaultMessage: 'Content list',
	},
	contentListDescription: {
		id: 'instance.settings.tabs.troubleshooting.content-list.description',
		defaultMessage:
			'Copy a link listing everything installed in this instance, handy when asking for help.',
	},
	copyContentList: {
		id: 'instance.settings.tabs.troubleshooting.content-list.copy',
		defaultMessage: 'Copy content list',
	},
	moreCopyOptions: {
		id: 'instance.settings.tabs.troubleshooting.content-list.more-options',
		defaultMessage: 'More list formats',
	},
	projectNames: {
		id: 'instance.settings.tabs.troubleshooting.content-list.project-names',
		defaultMessage: 'Project names',
	},
	fileNames: {
		id: 'instance.settings.tabs.troubleshooting.content-list.file-names',
		defaultMessage: 'File names',
	},
	projectLinks: {
		id: 'instance.settings.tabs.troubleshooting.content-list.project-links',
		defaultMessage: 'Project links',
	},
	markdownLinks: {
		id: 'instance.settings.tabs.troubleshooting.content-list.markdown-links',
		defaultMessage: 'Markdown links',
	},
	linkCopiedTitle: {
		id: 'instance.settings.tabs.troubleshooting.content-list.link-copied.title',
		defaultMessage: 'Content list link copied',
	},
	textCopiedTitle: {
		id: 'instance.settings.tabs.troubleshooting.content-list.text-copied.title',
		defaultMessage: 'Content list copied',
	},
	textCopiedText: {
		id: 'instance.settings.tabs.troubleshooting.content-list.text-copied.text',
		defaultMessage: "Couldn't create a link, so the list was copied as text instead.",
	},
	debugInfoTitle: {
		id: 'instance.settings.tabs.troubleshooting.debug-info.title',
		defaultMessage: 'Debug info',
	},
	debugInfoDescription: {
		id: 'instance.settings.tabs.troubleshooting.debug-info.description',
		defaultMessage: "Export this instance's logs and diagnostic data to a ZIP file to share.",
	},
	exportDebugInfo: {
		id: 'instance.settings.tabs.troubleshooting.debug-info.export',
		defaultMessage: 'Export debug info',
	},
})

const exportingContentList = ref(false)
const copiedContentList = ref(false)

async function copyContentListLink(format: ContentListFormat) {
	exportingContentList.value = true
	try {
		const url = await copyAsPaste(await getContentList(instance.value.id, { format }))
		addNotification(
			url
				? {
						type: 'success',
						title: formatMessage(messages.linkCopiedTitle),
						text: url,
						copyText: url,
					}
				: {
						type: 'success',
						title: formatMessage(messages.textCopiedTitle),
						text: formatMessage(messages.textCopiedText),
					},
		)
		copiedContentList.value = true
		setTimeout(() => {
			copiedContentList.value = false
		}, 1500)
	} catch (error) {
		handleError(error as Error)
	} finally {
		exportingContentList.value = false
	}
}

const formatOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'project-names',
		label: formatMessage(messages.projectNames),
		icon: TextCursorInputIcon,
		action: () => copyContentListLink('names'),
	},
	{
		id: 'file-names',
		label: formatMessage(messages.fileNames),
		icon: FileIcon,
		action: () => copyContentListLink('file-names'),
	},
	{
		id: 'project-links',
		label: formatMessage(messages.projectLinks),
		icon: LinkIcon,
		action: () => copyContentListLink('urls'),
	},
	{
		id: 'markdown-links',
		label: formatMessage(messages.markdownLinks),
		icon: CodeIcon,
		action: () => copyContentListLink('markdown'),
	},
])

const exportingDebugInfo = computed(() => isExportingDebugInfo(instance.value.id))

async function handleExportDebugInfo() {
	await exportDebugInfo(formatMessage(messages.exportDebugInfo), {
		id: instance.value.id,
		name: instance.value.name,
		iconPath: instance.value.icon_path ?? null,
	}).catch(handleError)
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.contentListTitle) }}
			</h2>
			<SplitButton
				class="w-fit"
				:options="formatOptions"
				:menu-label="formatMessage(messages.moreCopyOptions)"
				:disabled="exportingContentList"
				@click="copyContentListLink('detailed')"
			>
				<LoaderCircleIcon
					v-if="exportingContentList"
					class="motion-safe:animate-spin"
					aria-hidden="true"
				/>
				<CheckIcon v-else-if="copiedContentList" aria-hidden="true" />
				<CopyIcon v-else aria-hidden="true" />
				{{ formatMessage(messages.copyContentList) }}
			</SplitButton>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.contentListDescription) }}
			</p>
		</div>
		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.debugInfoTitle) }}
			</h2>
			<Button
				class="w-fit"
				:disabled="exportingDebugInfo"
				:loading="exportingDebugInfo"
				@click="handleExportDebugInfo"
			>
				<LoaderCircleIcon
					v-if="exportingDebugInfo"
					class="motion-safe:animate-spin"
					aria-hidden="true"
				/>
				<DownloadIcon v-else aria-hidden="true" />
				{{ formatMessage(messages.exportDebugInfo) }}
			</Button>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.debugInfoDescription) }}
			</p>
		</div>
	</div>
</template>
