<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	FolderSearchIcon,
	RotateCounterClockwiseIcon,
	SpinnerIcon,
	StarIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmModal,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	Table,
	type TableColumn,
	useVIntl,
} from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref, useTemplateRef } from 'vue'

const messages = defineMessages({
	title: {
		id: 'modpack-scan-modal.title',
		defaultMessage: 'Modpack Scan ({scanned}/{total} Files)',
	},
	scanAllFiles: {
		id: 'modpack-scan-modal.scan-all-files',
		defaultMessage: 'Scan All Files',
	},
	packFileName: {
		id: 'modpack-scan-modal.pack-file-name',
		defaultMessage: 'Pack File Name',
	},
	newFiles: {
		id: 'modpack-scan-modal.new-files',
		defaultMessage: 'New Files',
	},
	newGroups: {
		id: 'modpack-scan-modal.new-groups',
		defaultMessage: 'New Groups',
	},
	loadingVersions: {
		id: 'modpack-scan-modal.loading-versions',
		defaultMessage: 'Loading versions...',
	},
	noFiles: {
		id: 'modpack-scan-modal.no-files',
		defaultMessage: 'No files found.',
	},
	notScanned: {
		id: 'modpack-scan-modal.not-scanned',
		defaultMessage: 'Not scanned',
	},
	scanning: {
		id: 'modpack-scan-modal.scanning',
		defaultMessage: 'Scanning...',
	},
	failed: {
		id: 'modpack-scan-modal.failed',
		defaultMessage: 'Failed',
	},
	overrideFiles: {
		id: 'modpack-scan-modal.override-files',
		defaultMessage: 'Override Files ({count})',
	},
	loadVersionsError: {
		id: 'modpack-scan-modal.load-versions-error',
		defaultMessage: 'Failed to load versions: {error}',
	},
	scanError: {
		id: 'modpack-scan-modal.scan-error',
		defaultMessage: 'Some files failed to scan: \n\n{error}',
	},
	clearAllGroups: {
		id: 'modpack-scan-modal.clear-all-groups',
		defaultMessage: 'Clear All Groups',
	},
})

type ScanTableColumn = 'filename' | 'newFiles' | 'newGroups'

type ScanRow = {
	id: string
	filename: string
	primary: boolean
	scan?: Labrinth.Attribution.Internal.FileScanResponse
	isScanning: boolean
	error?: string
	newFiles?: number
	newGroups?: number
}

const props = defineProps<{
	project_id: string
}>()

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const modalRef = useTemplateRef<InstanceType<typeof NewModal>>('modalRef')
const clearModalRef = useTemplateRef<InstanceType<typeof ConfirmModal>>('clearModalRef')
const { formatMessage } = useVIntl()

const rows = ref<Record<string, ScanRow>>({})
const isLoadingVersions = ref(false)
const isScanning = ref(false)
const isClearing = ref(false)
const versionLoadError = ref<string | null>(null)
const scanError = ref<string | null>(null)
const requestId = ref(0)
const scanRequestId = ref(0)

const columns = computed<TableColumn<ScanTableColumn>[]>(() => [
	{ key: 'filename', label: formatMessage(messages.packFileName), width: '60%' },
	{ key: 'newFiles', label: formatMessage(messages.newFiles), align: 'center', width: '20%' },
	{ key: 'newGroups', label: formatMessage(messages.newGroups), align: 'center', width: '20%' },
])

const scannedCount = computed(
	() => Object.entries(rows.value).filter(([_, row]) => row.scan || row.error).length,
)
const isBusy = computed(() => isLoadingVersions.value || isScanning.value || isClearing.value)
const titleButtonsDisabled = computed(() => isBusy || Object.keys(rows.value).length === 0)
const rescanButtonsDisabled = computed(() => isLoadingVersions.value || isClearing.value)

const rowErrors = computed(() =>
	Object.entries(rows.value)
		.filter(([_, row]) => row.error)
		.map(([_, row]) => row),
)

const rowScanError = computed(() => {
	if (rowErrors.value.length === 0) return undefined
	return formatMessage(messages.scanError, {
		error: rowErrors.value.map((r) => `\n - ${r.filename}`).join(''),
	})
})

function getErrorMessage(error: unknown) {
	if (error instanceof Error) {
		return error.message
	}

	if (typeof error === 'object' && error !== null && 'data' in error) {
		const data = (error as { data?: { description?: string } }).data
		if (data?.description) {
			return data.description
		}
	}

	return String(error)
}

async function runWithConcurrency<T>(
	items: T[],
	limit: number,
	task: (item: T) => Promise<void>,
): Promise<void> {
	const queue = [...items]
	const workers = Array.from({ length: limit }, async () => {
		while (queue.length) {
			const item = queue.shift()
			if (item === undefined) return
			await task(item)
		}
	})
	await Promise.all(workers)
}

async function fetchAllVersions() {
	const currentRequestId = ++requestId.value
	isLoadingVersions.value = true
	versionLoadError.value = null
	scanError.value = null
	rows.value = {}

	try {
		const versions = await client.labrinth.versions_v2.getProjectVersions(props.project_id)
		if (currentRequestId !== requestId.value) {
			return
		}

		const filteredVersions = versions
			.flatMap((version) => version.files)
			.filter((file): file is Labrinth.Versions.v2.VersionFile & { id: string } => Boolean(file.id))

		for (const version of filteredVersions) {
			rows.value[version.id] = {
				id: version.id,
				filename: version.filename,
				primary: version.primary,
				isScanning: false,
				newFiles: undefined,
				newGroups: undefined,
			}
		}
	} catch (error) {
		if (currentRequestId === requestId.value) {
			versionLoadError.value = formatMessage(messages.loadVersionsError, {
				error: getErrorMessage(error),
			})
		}
	} finally {
		if (currentRequestId === requestId.value) {
			isLoadingVersions.value = false
		}
	}
}

async function fetchScan(id: string) {
	console.log(`scanning for row with id ${id}`)

	rows.value[id].isScanning = true
	try {
		if (Math.random() < 0.25) throw new Error('test error')
		const scan = await client.labrinth.attribution_internal.scanFile(id)

		rows.value[id].scan = scan
		rows.value[id].newFiles = scan.new_attribution_files
		rows.value[id].newGroups = scan.new_attribution_groups

		rows.value[id].error = undefined
	} catch (error) {
		rows.value[id].error = getErrorMessage(error)
		scanError.value = formatMessage(messages.scanError, { error: rows.value[id].error })
	} finally {
		rows.value[id].isScanning = false
	}
}

async function fetchAllScans() {
	if (isBusy.value) return

	isScanning.value = true
	scanError.value = null

	Object.entries(rows.value).map(([id, row]) => {
		rows.value[id] = {
			...row,
			scan: undefined,
			isScanning: false,
			error: undefined,
			newFiles: undefined,
			newGroups: undefined,
		}
	})

	try {
		await runWithConcurrency(Object.keys(rows.value), 10, async (id: string) => {
			await fetchScan(id)
		})
	} finally {
		isScanning.value = false
	}
}

function showConfirmClearGroups() {
	clearModalRef.value?.show()
}

async function clearAllGroups() {
	if (isBusy.value) {
		return
	}

	let failed = false

	try {
		isClearing.value = true
		const groups = await client.labrinth.attribution_internal.listProjectAttribution(
			props.project_id,
		)

		for (const group of groups) {
			await client.labrinth.attribution_internal.deleteGroup(group.id)
		}

		await queryClient.invalidateQueries({ queryKey: ['project-attribution', props.project_id] })
	} catch (error) {
		failed = true
		addNotification({
			type: 'error',
			title: 'An error occurred',
			text: `Failed to clear all groups: ${getErrorMessage(error)}`,
		})
	} finally {
		isClearing.value = false
	}

	if (!failed) {
		addNotification({
			type: 'success',
			title: 'Success',
			text: 'All groups cleared successfully.',
		})
	}
}

function show() {
	scanRequestId.value++
	isScanning.value = false
	rows.value = {}
	void fetchAllVersions()
	modalRef.value?.show()
}

function hide() {
	modalRef.value?.hide()
}
defineExpose({ show, hide })
</script>

<template>
	<ConfirmModal
		ref="clearModalRef"
		title="Clear all permission groups?"
		description="This will clear **all** groups for this project. This action cannot be undone."
		proceed-label="Clear"
		@proceed="clearAllGroups"
	/>

	<NewModal
		ref="modalRef"
		width="60vw"
		:close-on-click-outside="false"
		:close-on-esc="false"
		:disable-close="isBusy"
	>
		<template #title>
			<div class="flex w-full items-center justify-between gap-2">
				<span class="text-2xl font-semibold text-contrast">
					{{
						formatMessage(messages.title, {
							scanned: scannedCount,
							total: Object.keys(rows).length,
						})
					}}
				</span>
				<div class="flex items-center gap-2">
					<ButtonStyled circular color="red" color-fill="none">
						<button
							v-tooltip="formatMessage(messages.clearAllGroups)"
							:disabled="titleButtonsDisabled.value"
							@click="showConfirmClearGroups"
						>
							<TrashIcon v-if="!isClearing" aria-hidden="true" />
							<SpinnerIcon class="animate-spin" v-else />
						</button>
					</ButtonStyled>
					<ButtonStyled circular>
						<button
							v-tooltip="formatMessage(messages.scanAllFiles)"
							:disabled="titleButtonsDisabled.value"
							@click="fetchAllScans"
						>
							<FolderSearchIcon v-if="!isScanning" aria-hidden="true" />
							<SpinnerIcon class="animate-spin" v-else />
						</button>
					</ButtonStyled>
				</div>
			</div>
		</template>

		<div class="w-full">
			<div
				v-if="versionLoadError || rowScanError"
				class="mb-3 rounded-xl bg-highlight-red px-4 py-1 text-red"
			>
				<div v-html="renderString((versionLoadError || rowScanError) ?? '')"></div>
			</div>
			<Table
				:columns="columns"
				:data="Object.entries(rows).map(([_, row]) => row)"
				row-key="id"
				:row-below-visible="
					(row) => Boolean(row.scan?.scanned_file_names && row.scan.scanned_file_names.length > 0)
				"
				table-min-width="42rem"
			>
				<template #cell-filename="{ row }">
					<div class="flex min-w-0 items-center gap-1 text-contrast">
						<StarIcon v-if="row.primary" class="size-4 shrink-0" aria-hidden="true" />
						<span class="min-w-0 truncate">{{ row.filename }}</span>
					</div>
				</template>
				<template #cell-newFiles="{ row }">
					<span v-if="row.isScanning">{{ formatMessage(messages.scanning) }}</span>
					<span v-else-if="row.error" v-tooltip="row.error" class="flex justify-center">
						<ButtonStyled
							class="justify-self-center"
							color="red"
							type="outlined"
							hover-color-fill="background"
						>
							<button :disabled="rescanButtonsDisabled" @click="() => fetchScan(row.id)">
								<RotateCounterClockwiseIcon />
								{{ formatMessage(messages.failed) }}
							</button>
						</ButtonStyled>
					</span>
					<span v-else-if="row.scan">{{ row.scan.new_attribution_files }}</span>
					<span v-else>{{ formatMessage(messages.notScanned) }}</span>
				</template>
				<template #cell-newGroups="{ row }">
					<span v-if="row.isScanning">{{ formatMessage(messages.scanning) }}</span>
					<span v-else-if="row.error" v-tooltip="row.error" class="flex justify-center">
						<ButtonStyled
							class="justify-self-center"
							color="red"
							type="outlined"
							hover-color-fill="background"
						>
							<button :disabled="rescanButtonsDisabled" @click="() => fetchScan(row.id)">
								<RotateCounterClockwiseIcon />
								{{ formatMessage(messages.failed) }}
							</button>
						</ButtonStyled>
					</span>
					<span v-else-if="row.scan">{{ row.scan.new_attribution_groups }}</span>
					<span v-else>{{ formatMessage(messages.notScanned) }}</span>
				</template>
				<template #row-below="{ row }">
					<div class="border-0 border-t border-solid border-surface-4 px-4 py-3">
						<details>
							<summary>
								{{
									formatMessage(messages.overrideFiles, {
										count: row.scan?.scanned_file_names.length ?? 0,
									})
								}}
							</summary>
							<div class="flex flex-wrap gap-1 pt-2">
								<span
									v-for="name of row.scan?.scanned_file_names ?? []"
									:key="name"
									v-tooltip="name"
									class="flex items-center gap-1 text-wrap rounded-full bg-button-bg px-2 py-0.5 text-xs font-medium text-contrast"
								>
									{{ name }}
								</span>
							</div>
						</details>
					</div>
				</template>
				<template #empty-state>
					<div class="flex h-64 items-center justify-center text-secondary">
						{{ formatMessage(isLoadingVersions ? messages.loadingVersions : messages.noFiles) }}
					</div>
				</template>
			</Table>
		</div>
	</NewModal>
</template>
