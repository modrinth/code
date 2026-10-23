<template>
	<ArchiveDetailsSection
		:kind="formatMessage(messages.external)"
		:title="source.name"
		:icon="ExternalIcon"
		icon-class="text-secondary"
		details-key="project"
	>
		<template #actions>
			<div class="flex shrink-0 items-center gap-1">
				<ButtonLink
					v-tooltip="formatMessage(messages.download)"
					type="quiet"
					size="xs"
					:href="source.url"
					:download="source.name"
					:aria-label="formatMessage(messages.download)"
				>
					<DownloadIcon aria-hidden="true" />
				</ButtonLink>
			</div>
		</template>
		<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.description) }}</p>
	</ArchiveDetailsSection>
</template>

<script setup lang="ts">
import { DownloadIcon, ExternalIcon } from '@modrinth/assets'
import {
	ArchiveDetailsSection,
	type ArchiveSource,
	ButtonLink,
	defineMessages,
	useVIntl,
} from '@modrinth/ui'

/**
 * Stands in for the project at the top of the archive viewer's details when the archive was
 * opened from a link no project's version holds, named after the file the link points to.
 */
defineProps<{
	source: ArchiveSource
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	external: { id: 'archive.external-details.external', defaultMessage: 'External' },
	description: {
		id: 'archive.external-details.description',
		defaultMessage: "Opened from a link that isn't part of a project's versions.",
	},
	download: { id: 'archive.project-details.download', defaultMessage: 'Download file' },
})
</script>
