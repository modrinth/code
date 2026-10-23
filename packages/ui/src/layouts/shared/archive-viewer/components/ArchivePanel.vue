<template>
	<LogsPanel v-if="panel === 'logs'" />
	<SettingsPanel v-else-if="panel === SETTINGS_PANEL_ID" />
	<template v-else-if="current">
		<FileSideBar v-if="panel === 'tree'" fill-height :add-border="false" />
		<ArchiveViewerScope v-else :key="current.root.key" :archive="current">
			<DetailsPanel>
				<template v-if="slots['project-details']" #source>
					<slot name="project-details" />
				</template>
			</DetailsPanel>
		</ArchiveViewerScope>
	</template>
	<div v-else class="p-3">
		<ArchiveStatus>
			<template v-if="slots.empty" #empty>
				<slot name="empty" />
			</template>
		</ArchiveStatus>
	</div>
</template>

<script setup lang="ts">
import { defineComponent, type PropType } from 'vue'

import FileSideBar from '#ui/layouts/shared/files-tab/components/FileSideBar.vue'

import type { ArchivePanelId } from '../panels'
import { injectArchiveSession, type OpenArchive } from '../providers/archive-session'
import { provideArchiveViewer } from '../providers/archive-viewer'
import { SETTINGS_PANEL_ID } from '../settings'
import ArchiveStatus from './ArchiveStatus.vue'
import DetailsPanel from './panels/details/index.vue'
import LogsPanel from './panels/LogsPanel.vue'
import SettingsPanel from './panels/settings-panel.vue'

/**
 * The content of one of the archive viewer's panels (see `useArchivePanels`), within an archive
 * session. Panels about the archive show how opening it is going until it's open.
 */
defineProps<{
	panel: ArchivePanelId
}>()

const slots = defineSlots<{
	/** Details about where the archive comes from, shown at the top of the details panel. */
	'project-details'?: () => unknown
	/** Shown while there's no archive to open. */
	empty?: () => unknown
}>()

const { current } = injectArchiveSession()

/** Provides the open archive to the panels within, which read it once they're set up. */
const ArchiveViewerScope = defineComponent({
	name: 'ArchiveViewerScope',
	props: { archive: { type: Object as PropType<OpenArchive>, required: true } },
	setup(props, { slots: scopeSlots }) {
		provideArchiveViewer(props.archive)
		return () => scopeSlots.default?.()
	},
})
</script>
