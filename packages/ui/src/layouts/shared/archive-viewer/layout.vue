<template>
	<div class="flex h-full min-h-0 w-full flex-col">
		<ArchiveSession
			v-slot="{ opened }"
			:source="source"
			:path="path"
			:retain="retain"
			@update:path="(value) => emit('update:path', value)"
		>
			<ArchiveWorkbench
				v-if="opened"
				:storage-key="storageKey"
				:hidden-regions="hiddenRegions"
				:popout-url="popoutUrl"
			>
				<template #toolbar>
					<slot name="toolbar" />
					<ArchiveProgress />
				</template>
				<template #toolbar-end>
					<ArchiveSettings />
				</template>
				<template v-if="slots['project-details']" #project-details>
					<slot name="project-details" />
				</template>
				<template v-if="slots.empty" #empty>
					<slot name="empty" />
				</template>
			</ArchiveWorkbench>
			<template v-else>
				<div
					class="flex min-h-10 shrink-0 flex-wrap items-center gap-2 border-0 border-b border-solid border-surface-5 bg-surface-3 px-2 py-1"
				>
					<slot name="toolbar" />
				</div>
				<div class="min-h-0 flex-1 overflow-auto p-4">
					<ArchiveStatus>
						<template v-if="slots.empty" #empty>
							<slot name="empty" />
						</template>
					</ArchiveStatus>
				</div>
			</template>
		</ArchiveSession>
	</div>
</template>

<script setup lang="ts">
import type { VNode } from 'vue'

import type { WorkbenchRegion } from '#ui/layouts/shared/workbench/types'

import ArchiveSettings from './components/archive-settings-button.vue'
import ArchiveProgress from './components/ArchiveProgress.vue'
import ArchiveSession from './components/ArchiveSession.vue'
import ArchiveStatus from './components/ArchiveStatus.vue'
import ArchiveWorkbench from './components/ArchiveWorkbench.vue'
import type { ArchiveSource } from './types'

withDefaults(
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
		/**
		 * An empty page of the same origin, which lets panels be popped out into windows loading it.
		 * Without one, panels stay within the page.
		 */
		popoutUrl?: string
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
</script>
