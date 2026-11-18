<script setup lang="ts">
import { CheckIcon, PlusIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	NewModal,
	Table,
	type TableColumn,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import type { JavaVersion } from '@/platform/app-lib/jre/types'

import { detectedJresQueryOptions } from '../queries'

const emit = defineEmits<{
	submit: [value: JavaVersion]
}>()

const { formatMessage } = useVIntl()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const open = ref(false)
const version = ref<number | null>(null)
const currentPath = ref('')

const detectedJresQuery = useQuery(
	computed(() => ({ ...detectedJresQueryOptions(version.value), enabled: open.value })),
)
const detectedJres = computed(() => detectedJresQuery.data.value ?? [])

const columns = computed<TableColumn<'version' | 'path' | 'actions'>[]>(() => [
	{ key: 'version', label: formatMessage(messages.versionColumn), width: '9rem' },
	{ key: 'path', label: formatMessage(messages.pathColumn) },
	{
		key: 'actions',
		label: formatMessage(messages.actionsColumn),
		align: 'right',
		width: '10rem',
	},
])

function show(javaVersion: number | null, current?: Partial<JavaVersion> | null) {
	version.value = javaVersion
	currentPath.value = current?.path ?? ''
	open.value = true
	modal.value?.show()
}

function select(jre: JavaVersion) {
	emit('submit', jre)
	modal.value?.hide()
	trackEvent('JavaAutoDetect', { path: jre.path, version: jre.version })
}

defineExpose({ show })

const messages = defineMessages({
	title: {
		id: 'app.java-detection.title',
		defaultMessage: 'Select Java installation',
	},
	versionColumn: {
		id: 'app.java-detection.columns.version',
		defaultMessage: 'Version',
	},
	pathColumn: {
		id: 'app.java-detection.columns.path',
		defaultMessage: 'Path',
	},
	actionsColumn: {
		id: 'app.java-detection.columns.actions',
		defaultMessage: 'Actions',
	},
	selected: {
		id: 'app.java-detection.selected',
		defaultMessage: 'Selected',
	},
	select: {
		id: 'app.java-detection.select',
		defaultMessage: 'Select',
	},
	noInstallationsFound: {
		id: 'app.java-detection.no-installations-found',
		defaultMessage: 'No Java installations found.',
	},
	cancel: {
		id: 'app.java-detection.cancel',
		defaultMessage: 'Cancel',
	},
})
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)" :on-hide="() => (open = false)">
		<Table :columns="columns" :data="detectedJres" row-key="path">
			<template #cell-version="{ value }">
				<span class="font-semibold text-primary">{{ value }}</span>
			</template>
			<template #cell-path="{ value }">
				<span v-tooltip="value" class="block truncate font-mono text-xs">{{ value }}</span>
			</template>
			<template #cell-actions="{ row }">
				<div class="flex items-center justify-end">
					<Button v-if="currentPath === row.path" disabled>
						<CheckIcon aria-hidden="true" />
						{{ formatMessage(messages.selected) }}
					</Button>
					<Button v-else @click="select(row)">
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.select) }}
					</Button>
				</div>
			</template>
			<template #empty-state>
				<div
					v-if="detectedJresQuery.isFetching.value"
					role="status"
					class="flex items-center gap-2 p-4 text-secondary"
				>
					<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
					{{ formatMessage(commonMessages.loadingLabel) }}
				</div>
				<div v-else-if="detectedJresQuery.isError.value" class="p-4">
					<Button @click="detectedJresQuery.refetch()">
						{{ formatMessage(commonMessages.refreshButton) }}
					</Button>
				</div>
				<div v-else class="p-4 text-secondary">
					{{ formatMessage(messages.noInstallationsFound) }}
				</div>
			</template>
		</Table>
		<template #actions>
			<div class="flex justify-end">
				<Button type="outlined" class="!border !border-surface-4" @click="modal?.hide()">
					<XIcon aria-hidden="true" />
					{{ formatMessage(messages.cancel) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
