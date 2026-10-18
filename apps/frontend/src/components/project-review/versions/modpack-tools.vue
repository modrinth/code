<template>
	<template v-if="project?.project_types.includes('modpack')">
		<ModpackScanModal ref="scanModal" :project_id="project.id" />
		<TeleportOverflowMenu
			type="quiet"
			:label="formatMessage(messages.tools)"
			:options="actions"
			:icon-only="false"
			:circular="false"
			:disabled="isLoading || !!error || pending"
		>
			{{ formatMessage(messages.tools) }}
			<ChevronDownIcon aria-hidden="true" />
		</TeleportOverflowMenu>
	</template>
</template>

<script setup lang="ts">
import { ChevronDownIcon, FileArchiveIcon, FolderSearchIcon, ScanEyeIcon } from '@modrinth/assets'
import {
	commonMessages,
	defineMessages,
	injectLoadingState,
	injectModrinthClient,
	injectNotificationManager,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, useTemplateRef } from 'vue'

import ModpackScanModal from '~/components/ui/moderation/ModpackScanModal.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

const emit = defineEmits<{
	'scan-ai': [urls: string[]]
}>()
const { project, versions, versionsQuery, isLoading, error } = injectProjectReviewPageContext()
const { pending } = injectReviewSubmission()
const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const loadingState = injectLoadingState()
const scanModal = useTemplateRef<InstanceType<typeof ModpackScanModal>>('scanModal')
const isCheckingModpackArchives = ref(false)
const messages = defineMessages({
	tools: {
		id: 'project-review.versions.modpack-tools',
		defaultMessage: 'Modpack tools',
	},
	viewC2paInfo: {
		id: 'project-review.versions.view-c2pa-info',
		defaultMessage: 'View C2PA Info',
	},
	rescanModpack: {
		id: 'project.actions.rescan-modpack',
		defaultMessage: 'Rescan modpack',
	},
	checkModpackArchives: {
		id: 'project.actions.check-modpack-archives',
		defaultMessage: 'Check modpack unzip',
	},
	checkingModpackArchives: {
		id: 'project.actions.checking-modpack-archives',
		defaultMessage: 'Checking...',
	},
	checkModpackArchivesSuccess: {
		id: 'project.notification.check-modpack-archives.success',
		defaultMessage:
			'{count, plural, one {The modpack file can be unzipped.} other {All # modpack files can be unzipped.}}',
	},
	checkModpackArchivesFailed: {
		id: 'project.notification.check-modpack-archives.failed',
		defaultMessage: 'Some modpack files could not be unzipped',
	},
	checkModpackArchivesNoFiles: {
		id: 'project.notification.check-modpack-archives.no-files',
		defaultMessage: 'No .mrpack files were found for this project.',
	},
})
const c2paUrls = computed(() => [
	...new Set(versions.value.flatMap((version) => version.files.map((file) => file.url))),
])
const actions = computed(() => [
	{
		id: 'rescan-modpack',
		label: formatMessage(messages.rescanModpack),
		icon: FolderSearchIcon,
		tone: 'orange' as const,
		action: () => scanModal.value?.show(),
	},
	{
		id: 'check-modpack-archives',
		label: formatMessage(
			isCheckingModpackArchives.value
				? messages.checkingModpackArchives
				: messages.checkModpackArchives,
		),
		icon: FileArchiveIcon,
		tone: 'orange' as const,
		disabled: isCheckingModpackArchives.value,
		action: checkModpackArchives,
	},
	{
		id: 'view-c2pa-info',
		label: formatMessage(messages.viewC2paInfo),
		icon: ScanEyeIcon,
		tone: 'orange' as const,
		disabled:
			versionsQuery.isPending.value || versionsQuery.isError.value || !c2paUrls.value.length,
		action: () => emit('scan-ai', c2paUrls.value),
	},
])

async function checkModpackArchives() {
	if (!project.value || isCheckingModpackArchives.value) return

	isCheckingModpackArchives.value = true
	const loadingToken = loadingState.begin()

	try {
		const versions = await client.labrinth.versions_v2.getProjectVersions(project.value.id)
		const filesByUrl = new Map(
			versions
				.flatMap((version) => version.files)
				.filter((file) => file.filename.toLowerCase().endsWith('.mrpack'))
				.map((file) => [file.url, file]),
		)
		const files = [...filesByUrl.values()]

		if (files.length === 0) {
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: formatMessage(messages.checkModpackArchivesNoFiles),
				type: 'error',
			})
			return
		}

		const { default: JSZip } = await import('jszip')
		const failures: { filename: string; error: string }[] = []

		for (const file of files) {
			try {
				const response = await fetch(file.url)
				if (!response.ok) {
					throw new Error(`Download failed (${response.status} ${response.statusText})`)
				}

				await JSZip.loadAsync(await response.blob(), { checkCRC32: true })
			} catch (error) {
				failures.push({
					filename: file.filename,
					error: error instanceof Error ? error.message : String(error),
				})
			}
		}

		if (failures.length > 0) {
			addNotification({
				title: formatMessage(messages.checkModpackArchivesFailed),
				text: failures.map((failure) => `${failure.filename}: ${failure.error}`).join('\n'),
				type: 'error',
			})
			return
		}

		addNotification({
			title: formatMessage(commonMessages.successLabel),
			text: formatMessage(messages.checkModpackArchivesSuccess, { count: files.length }),
			type: 'success',
		})
	} catch (error) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		})
	} finally {
		isCheckingModpackArchives.value = false
		loadingState.end(loadingToken)
	}
}
</script>
