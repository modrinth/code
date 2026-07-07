<template>
	<div v-if="downloadRows.length > 0" class="flex flex-col gap-1">
		<div v-if="showTitle" class="flex flex-wrap items-center justify-between gap-2">
			<h3 class="m-0 flex items-center gap-1.5 text-base font-semibold text-contrast">
				{{ sectionTitle }}
				<InfoIcon
					v-if="duplicateDependencyRowsHidden"
					v-tooltip="formatMessage(messages.duplicateDependenciesHidden)"
					aria-hidden="true"
					class="size-4 text-secondary"
				/>
			</h3>
		</div>
		<div class="flex flex-col gap-2">
			<DownloadDependency
				v-for="dependency in downloadRows"
				:key="dependency.key"
				:dependency="dependency"
				@download="emit('download')"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import DownloadDependency from './DownloadDependency.vue'
import { injectDownloadModalProvider } from './download-modal-provider'

defineOptions({
	name: 'DownloadDependencies',
})

const props = withDefaults(
	defineProps<{
		showTitle?: boolean
	}>(),
	{
		showTitle: true,
	},
)

const emit = defineEmits<{
	download: []
}>()
const { formatMessage } = useVIntl()
const {
	visibleDependencyRows,
	duplicateDependencyRowsHidden,
	downloadRows,
} = injectDownloadModalProvider()

const sectionTitle = computed(() =>
	formatMessage(
		visibleDependencyRows.value.length > 0
			? messages.dependenciesTitle
			: messages.additionalFilesTitle,
	),
)

const messages = defineMessages({
	dependenciesTitle: {
		id: 'project.download.dependencies-title',
		defaultMessage: 'Dependencies',
	},
	duplicateDependenciesHidden: {
		id: 'project.download.duplicate-dependencies-hidden',
		defaultMessage: 'Duplicate dependencies are hidden',
	},
	additionalFilesTitle: {
		id: 'project.download.additional-files-title',
		defaultMessage: 'Additional files',
	},
})
</script>
