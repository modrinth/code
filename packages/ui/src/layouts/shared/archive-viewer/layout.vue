<template>
	<div class="flex h-full min-h-0 w-full flex-col">
		<ArchiveWorkbench
			v-if="stage === 'ready' && root && handler && source"
			:key="root.key"
			:root="root"
			:handler="handler"
			:source="source"
			:path="path"
			:storage-key="storageKey"
			:hidden-regions="hiddenRegions"
			@navigate="(value) => emit('update:path', value)"
		>
			<template #toolbar>
				<slot name="toolbar" />
				<div
					v-if="!fullyLoaded || fetching"
					class="flex items-center gap-1.5 text-xs text-secondary"
					role="status"
				>
					<SpinnerIcon class="size-3.5 animate-spin" aria-hidden="true" />
					{{ formatMessage(fullyLoaded ? messages.fetchingRest : messages.readingNested) }}
					<span v-if="fetchedLabel" class="tabular-nums">({{ fetchedLabel }})</span>
				</div>
			</template>
			<template v-if="slots['project-details']" #project-details>
				<slot name="project-details" />
			</template>
		</ArchiveWorkbench>
		<template v-else>
			<div
				class="flex min-h-10 shrink-0 flex-wrap items-center gap-2 border-0 border-b border-solid border-surface-5 bg-surface-3 px-2 py-1"
			>
				<slot name="toolbar" />
			</div>
			<div class="min-h-0 flex-1 overflow-auto p-4">
				<Admonition
					v-if="stage === 'error'"
					type="critical"
					:header="formatMessage(messages.errorTitle, { name: source?.name ?? '' })"
					:body="error?.message"
				>
					<template #actions>
						<Button @click="retry">
							<RefreshCwIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.retryButton) }}
						</Button>
					</template>
				</Admonition>
				<Admonition
					v-else-if="stage === 'reading'"
					type="info"
					:header="formatMessage(messages.reading, { name: source?.name ?? '' })"
					:body="formatMessage(messages.readingDescription)"
					waiting
				/>
				<slot v-else name="empty" />
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import { RefreshCwIcon, SpinnerIcon } from '@modrinth/assets'
import { computed, toRef, type VNode } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import { useFormatBytes } from '#ui/composables/format-bytes'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import type { WorkbenchRegion } from '#ui/layouts/shared/workbench/types'
import { commonMessages } from '#ui/utils/common-messages'

import ArchiveWorkbench from './components/ArchiveWorkbench.vue'
import { useArchiveLoader } from './composables/archive-loader'
import type { ArchiveSource } from './types'

const props = withDefaults(
	defineProps<{
		source: ArchiveSource | null
		/** Path of the entry open within the archive, without a leading slash. */
		path?: string | null
		/** Keeps the archive open in the worker after leaving it, so reopening it skips reading it. */
		retain?: boolean
		/**
		 * Where the panel layout is persisted. `null` keeps it in memory only, e.g. for an embedded
		 * view that shouldn't change the user's own arrangement.
		 */
		storageKey?: string | null
		/** Regions hidden when opened, regardless of how they were left. */
		hiddenRegions?: WorkbenchRegion[]
	}>(),
	{ storageKey: 'archive-viewer' },
)

const emit = defineEmits<{
	'update:path': [path: string]
}>()

const slots = defineSlots<{
	/** Controls shown at the start of the toolbar, e.g. picking the archive to open. */
	toolbar?: () => VNode[]
	/** Details about where the archive comes from, shown as a panel when given. */
	'project-details'?: () => VNode[]
	/** Shown while there's no archive to open. */
	empty?: () => VNode[]
}>()

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()

const messages = defineMessages({
	reading: {
		id: 'archive-viewer.reading',
		defaultMessage: 'Reading {name}',
	},
	readingDescription: {
		id: 'archive-viewer.reading-description',
		defaultMessage: "Fetching the archive's index. Files are fetched as they are opened.",
	},
	readingNested: {
		id: 'archive-viewer.reading-nested',
		defaultMessage: 'Reading nested archives...',
	},
	fetchingRest: {
		id: 'archive-viewer.fetching-rest',
		defaultMessage: 'Fetching the rest of the archive...',
	},
	fetchedProgress: {
		id: 'archive-viewer.fetched-progress',
		defaultMessage: '{loaded} of {total} fetched',
	},
	errorTitle: {
		id: 'archive-viewer.error-title',
		defaultMessage: 'Unable to open {name}',
	},
})

const { stage, progress, error, root, handler, fullyLoaded, retry } = useArchiveLoader(
	toRef(props, 'source'),
	{ retain: props.retain },
)

/** Whether the rest of the archive is still being fetched in the background. */
const fetching = computed(() => {
	const { loaded, total } = progress.value
	return total != null && loaded < total
})

const fetchedLabel = computed(() => {
	const { loaded, total } = progress.value
	if (!loaded || !total) return null
	return formatMessage(messages.fetchedProgress, {
		loaded: formatBytes(loaded),
		total: formatBytes(total),
	})
})
</script>
