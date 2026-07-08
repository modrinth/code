<template>
	<div v-if="downloadRows.length > 0 || recommendedRows.length > 0" class="flex flex-col gap-4">
		<div v-if="downloadRows.length > 0" class="flex flex-col gap-2.5">
			<div v-if="showTitle" class="flex flex-wrap items-center justify-between gap-2">
				<h3 class="m-0 flex items-center gap-1.5 text-base font-semibold text-contrast">
					{{ formatMessage(messages.dependenciesTitle) }}
					<InfoIcon
						v-if="duplicateDependencyRowsHidden"
						v-tooltip="formatMessage(messages.duplicateDependenciesHidden)"
						aria-hidden="true"
						class="size-4 text-secondary"
					/>
				</h3>
			</div>
			<Admonition v-if="requiredResourcePackAdmonitionVisible" type="info">
				<IntlFormatted :message-id="messages.requiredResourcePackAdmonition">
					<template #folder>
						<code class="text-sm">resourcepacks</code class="text-sm">
					</template>
				</IntlFormatted>
			</Admonition>
			<Admonition v-if="dependencyResourcePackAdmonitionVisible" type="info">
				<IntlFormatted :message-id="messages.dependencyResourcePackAdmonition">
					<template #folder>
						<code class="text-sm">resourcepacks</code class="text-sm">
					</template>
				</IntlFormatted>
			</Admonition>
			<div class="rounded-2xl bg-surface-2 p-2 pl-4 pr-3">
				<DownloadDependency
					v-for="dependency in downloadRows"
					:key="dependency.key"
					:dependency="dependency"
					@download="emit('download')"
				/>
			</div>
		</div>

		<div v-if="recommendedRows.length > 0" class="flex flex-col gap-2.5">
			<h3 class="m-0 text-base font-semibold text-contrast">
				{{ formatMessage(messages.recommendedTitle) }}
			</h3>
			<div class="rounded-2xl bg-surface-2 p-2 pl-4 pr-3">
				<DownloadDependency
					v-for="dependency in recommendedRows"
					:key="dependency.key"
					:dependency="dependency"
					@download="emit('download')"
				/>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { InfoIcon } from '@modrinth/assets'
import { Admonition, defineMessages, IntlFormatted, useVIntl } from '@modrinth/ui'

import { injectDownloadModalProvider } from './download-modal-provider'
import DownloadDependency from './DownloadDependency.vue'

defineOptions({
	name: 'DownloadDependencies',
})

withDefaults(
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
	dependencyResourcePackAdmonitionVisible,
	duplicateDependencyRowsHidden,
	downloadRows,
	recommendedRows,
	requiredResourcePackAdmonitionVisible,
} = injectDownloadModalProvider()

const messages = defineMessages({
	dependenciesTitle: {
		id: 'project.download.dependencies-title',
		defaultMessage: 'Dependencies',
	},
	recommendedTitle: {
		id: 'project.download.recommended-title',
		defaultMessage: 'Recommended',
	},
	duplicateDependenciesHidden: {
		id: 'project.download.duplicate-dependencies-hidden',
		defaultMessage: 'Duplicate dependencies are hidden',
	},
	requiredResourcePackAdmonition: {
		id: 'project.download.required-resource-pack-admonition',
		defaultMessage:
			'This data pack also requires a resource pack. Download it and place it in your {folder} folder.',
	},
	dependencyResourcePackAdmonition: {
		id: 'project.download.dependency-resource-pack-admonition',
		defaultMessage:
			'This project has a dependency with a required resource pack. Download it and place it in your {folder} folder.',
	},
})
</script>
