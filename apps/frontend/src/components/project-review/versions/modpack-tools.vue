<template>
	<template v-if="project?.project_types.includes('modpack')">
		<ModpackScanModal ref="scanModal" :project_id="project.id" />
		<TeleportOverflowMenu
			type="quiet"
			label="Modpack tools"
			:options="actions"
			:icon-only="false"
			:circular="false"
			:disabled="isLoading || !!error || pending"
		>
			Modpack tools
			<ChevronDownIcon aria-hidden="true" />
		</TeleportOverflowMenu>
	</template>
</template>

<script setup lang="ts">
import { ChevronDownIcon, FileArchiveIcon, FolderSearchIcon, ScanEyeIcon } from '@modrinth/assets'
import {
	injectLoadingState,
	injectModrinthClient,
	injectNotificationManager,
	TeleportOverflowMenu,
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

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const loadingState = injectLoadingState()
const scanModal = useTemplateRef<InstanceType<typeof ModpackScanModal>>('scanModal')
const isCheckingModpackArchives = ref(false)

const c2paUrls = computed(() => [
	...new Set(versions.value.flatMap((version) => version.files.map((file) => file.url))),
])
const actions = computed(() => [
	{
		id: 'rescan-modpack',
		label: 'Rescan modpack',
		icon: FolderSearchIcon,
		tone: 'orange' as const,
		action: () => scanModal.value?.show(),
	},
	{
		id: 'check-modpack-archives',
		label: isCheckingModpackArchives.value ? 'Checking...' : 'Check modpack unzip',
		icon: FileArchiveIcon,
		tone: 'orange' as const,
		disabled: isCheckingModpackArchives.value,
		action: checkModpackArchives,
	},
	{
		id: 'view-c2pa-info',
		label: 'View C2PA Info',
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
				title: 'An error occurred',
				text: 'No .mrpack files were found for this project.',
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
				title: 'Some modpack files could not be unzipped',
				text: failures.map((failure) => `${failure.filename}: ${failure.error}`).join('\n'),
				type: 'error',
			})
			return
		}

		addNotification({
			title: 'Success',
			text:
				files.length === 1
					? 'The modpack file can be unzipped.'
					: `All ${files.length} modpack files can be unzipped.`,
			type: 'success',
		})
	} catch (error) {
		addNotification({
			title: 'An error occurred',
			text: error instanceof Error ? error.message : String(error),
			type: 'error',
		})
	} finally {
		isCheckingModpackArchives.value = false
		loadingState.end(loadingToken)
	}
}
</script>
