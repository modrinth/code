<script setup>
import { DropdownIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Collapsible,
	commonMessages,
	defineMessages,
	FileTreeSelect,
	injectNotificationManager,
	MarkdownEditor,
	NewModal,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { save } from '@tauri-apps/plugin-dialog'
import { readDir, stat } from '@tauri-apps/plugin-fs'
import { Tooltip } from 'floating-vue'
import { computed, nextTick, ref } from 'vue'

import { PackageIcon } from '@/assets/icons'
import {
	export_instance_mrpack,
	get_full_path,
	get_pack_export_candidates,
} from '@/helpers/instance'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: { id: 'app.export-modal.header', defaultMessage: 'Export modpack' },
	modpackNameLabel: { id: 'app.export-modal.modpack-name-label', defaultMessage: 'Modpack name' },
	modpackNamePlaceholder: {
		id: 'app.export-modal.modpack-name-placeholder',
		defaultMessage: 'Modpack name',
	},
	versionNumberLabel: {
		id: 'app.export-modal.version-number-label',
		defaultMessage: 'Version number',
	},
	versionNumberPlaceholder: {
		id: 'app.export-modal.version-number-placeholder',
		defaultMessage: '1.0.0',
	},
	descriptionPlaceholder: {
		id: 'app.export-modal.description-placeholder',
		defaultMessage: 'Enter modpack description...',
	},
	selectFilesLabel: {
		id: 'app.export-modal.select-files-label',
		defaultMessage: 'Configure included files',
	},
	exportButton: { id: 'app.export-modal.export-button', defaultMessage: 'Export' },
	fileSelectedSingular: {
		id: 'app.export-modal.file-selected-singular',
		defaultMessage: 'file selected',
	},
	filesSelectedPlural: {
		id: 'app.export-modal.files-selected-plural',
		defaultMessage: 'files selected',
	},
	selectedFilesTooltipTitle: {
		id: 'app.export-modal.selected-files-tooltip-title',
		defaultMessage: '{count, plural, one {# selected} other {# selected}}',
	},
})

const props = defineProps({
	instance: {
		type: Object,
		required: true,
	},
})

defineExpose({
	show: () => {
		resetExportState()
		exportModal.value.show()
		initFiles()
	},
})

const exportModal = ref(null)
const nameInput = ref(props.instance.name)
const exportDescription = ref('')
const versionInput = ref('1.0.0')
const files = ref([])
const selectedFilePaths = ref([])
const filesCollapsed = ref(true)
const fileTreeKey = ref(0)
const filesLoadId = ref(0)

const initFiles = async () => {
	const loadId = ++filesLoadId.value
	const [filePaths, instanceRoot] = await Promise.all([
		get_pack_export_candidates(props.instance.id),
		get_full_path(props.instance.id),
	])
	const expandedFiles = await Promise.all(
		filePaths.map((path) => expandExportCandidate(instanceRoot, path)),
	)
	if (loadId !== filesLoadId.value) return

	files.value = expandedFiles.flat()
	selectedFilePaths.value = files.value
		.filter(
			(file) =>
				!file.disabled &&
				(file.path.startsWith('mods') ||
					file.path.startsWith('datapacks') ||
					file.path.startsWith('resourcepacks') ||
					file.path.startsWith('shaderpacks') ||
					file.path.startsWith('config')),
		)
		.map((file) => file.path)
}

await initFiles()

const selectableFiles = computed(() => files.value.filter((file) => !file.disabled))

const selectedFileCount = computed(() => selectedFilePaths.value.length)

const selectedFileSummary = computed(() =>
	compressSelectedPaths(selectedFilePaths.value, selectableFiles.value),
)

const selectedFileCountLabel = computed(() =>
	selectedFileCount.value === 1
		? formatMessage(messages.fileSelectedSingular)
		: formatMessage(messages.filesSelectedPlural),
)

const exportPack = async () => {
	const outputPath = await save({
		defaultPath: `${nameInput.value} ${versionInput.value}.mrpack`,
		filters: [
			{
				name: 'Modrinth Modpack',
				extensions: ['mrpack'],
			},
		],
	})

	if (outputPath) {
		export_instance_mrpack(
			props.instance.id,
			outputPath,
			selectedFilePaths.value,
			versionInput.value,
			exportDescription.value,
			nameInput.value,
		).catch((err) => handleError(err))
		exportModal.value.hide()
	}
}

function resetExportState() {
	nameInput.value = props.instance.name
	exportDescription.value = ''
	versionInput.value = '1.0.0'
	files.value = []
	selectedFilePaths.value = []
	filesCollapsed.value = true
	fileTreeKey.value += 1
}

const scrollFilesIntoView = async (delay = 0) => {
	await nextTick()
	const scrollToBottom = () => requestAnimationFrame(() => exportModal.value?.scrollToBottom())
	if (delay > 0) {
		window.setTimeout(scrollToBottom, delay)
		return
	}
	scrollToBottom()
}

const toggleFilesCollapsed = () => {
	filesCollapsed.value = !filesCollapsed.value
	if (!filesCollapsed.value) {
		scrollFilesIntoView(320)
	}
}

function compressSelectedPaths(selectedPaths, allFiles) {
	const selected = new Set(selectedPaths)

	function summarizeFolder(folderPath) {
		const directChildren = getDirectChildren(folderPath, allFiles)
		const summaries = []

		for (const child of directChildren) {
			if (child.type === 'file') {
				if (selected.has(child.path)) {
					summaries.push(child.path)
				}
			} else {
				const descendants = allFiles.filter((file) => file.path.startsWith(`${child.path}/`))
				if (selected.has(child.path)) {
					summaries.push(`${child.path}/**`)
				} else if (descendants.length > 0 && descendants.every((file) => selected.has(file.path))) {
					summaries.push(`${child.path}/**`)
				} else {
					summaries.push(...summarizeFolder(child.path))
				}
			}
		}

		return summaries
	}

	return summarizeFolder('')
}

function getDirectChildren(folderPath, paths) {
	const children = new Map()
	const prefix = folderPath ? `${folderPath}/` : ''

	for (const item of paths) {
		const path = item.path
		if (prefix && !path.startsWith(prefix)) continue

		const relativePath = prefix ? path.slice(prefix.length) : path
		const segments = relativePath.split('/')
		const name = segments[0]
		const childPath = prefix ? `${prefix}${name}` : name

		if (segments.length === 1) {
			const type = item.type === 'directory' ? 'directory' : 'file'
			const existing = children.get(childPath)
			if (!existing || type === 'directory') {
				children.set(childPath, { type, path: childPath })
			}
		} else if (!children.has(childPath)) {
			children.set(childPath, { type: 'directory', path: childPath })
		}
	}

	return [...children.values()].sort((a, b) => {
		if (a.type !== b.type) return a.type === 'directory' ? -1 : 1
		return a.path.localeCompare(b.path, undefined, { numeric: true, sensitivity: 'base' })
	})
}

async function expandExportCandidate(instanceRoot, path) {
	try {
		const entries = await readDir(`${instanceRoot}/${path}`)
		if (entries.length === 0) {
			const metadata = await getExportCandidateMetadata(instanceRoot, path)
			return [
				{
					path,
					type: 'directory',
					disabled: true,
					modified: metadata.modified,
					count: 0,
				},
			]
		}

		const children = await Promise.all(
			entries.map(async (entry) => {
				const childPath = `${path}/${entry.name}`
				if (entry.isDirectory) {
					return expandExportCandidate(instanceRoot, childPath)
				}

				const metadata = await getExportCandidateMetadata(instanceRoot, childPath)
				return [
					{
						path: childPath,
						type: 'file',
						disabled: isExportCandidateDisabled(childPath),
						size: metadata.size,
						modified: metadata.modified,
					},
				]
			}),
		)
		return children.flat()
	} catch {
		const metadata = await getExportCandidateMetadata(instanceRoot, path)
		return [
			{
				path,
				type: 'file',
				disabled: isExportCandidateDisabled(path),
				size: metadata.size,
				modified: metadata.modified,
			},
		]
	}
}

async function getExportCandidateMetadata(instanceRoot, path) {
	try {
		const metadata = await stat(`${instanceRoot}/${path}`)
		return {
			size: metadata.size,
			modified: metadata.mtime ? Math.floor(metadata.mtime.getTime() / 1000) : undefined,
		}
	} catch {
		return {}
	}
}

function isExportCandidateDisabled(path) {
	return (
		path === 'profile.json' ||
		path.startsWith('modrinth_logs') ||
		path.startsWith('.fabric') ||
		path.startsWith('__MACOSX')
	)
}
</script>

<template>
	<NewModal
		ref="exportModal"
		:header="formatMessage(messages.header)"
		scrollable
		width="46rem"
		max-width="calc(100vw - 2rem)"
	>
		<div class="flex flex-col gap-4">
			<div class="grid grid-cols-2 gap-4">
				<div class="labeled_input w-full">
					<p class="text-contrast font-semibold">{{ formatMessage(messages.modpackNameLabel) }}</p>
					<StyledInput
						v-model="nameInput"
						type="text"
						:placeholder="formatMessage(messages.modpackNamePlaceholder)"
						clearable
						wrapper-class="w-full"
					/>
				</div>
				<div class="labeled_input w-full">
					<p class="text-contrast font-semibold">
						{{ formatMessage(messages.versionNumberLabel) }}
					</p>
					<StyledInput
						v-model="versionInput"
						type="text"
						:placeholder="formatMessage(messages.versionNumberPlaceholder)"
						clearable
						wrapper-class="w-full"
					/>
				</div>
			</div>
			<div class="flex flex-col gap-2 min-w-0">
				<p class="m-0 text-contrast font-semibold">
					{{ formatMessage(commonMessages.descriptionLabel) }}
				</p>
				<MarkdownEditor
					v-model="exportDescription"
					:placeholder="formatMessage(messages.descriptionPlaceholder)"
					:min-height="120"
					:max-height="240"
				/>
			</div>
			<div class="flex min-w-0 flex-col gap-3">
				<div class="flex w-full min-w-0 items-center justify-between gap-3">
					<button
						type="button"
						class="group m-0 flex w-full min-w-0 cursor-pointer items-center justify-between gap-3 border-none bg-transparent p-0 text-left"
						:aria-expanded="!filesCollapsed"
						@click="toggleFilesCollapsed"
					>
						<span class="min-w-0 truncate font-semibold text-contrast">
							{{ formatMessage(messages.selectFilesLabel) }}
						</span>
						<DropdownIcon
							class="size-5 shrink-0 text-contrast transition-transform duration-300"
							:class="{ 'rotate-180': !filesCollapsed }"
						/>
					</button>
				</div>
				<Collapsible :collapsed="filesCollapsed" overflow-visible>
					<FileTreeSelect
						:key="fileTreeKey"
						v-model="selectedFilePaths"
						class="min-w-0"
						:items="files"
						@navigate="scrollFilesIntoView()"
					>
						<template #actions>
							<span class="shrink-0 text-sm font-medium text-secondary">
								<Tooltip
									class="inline-flex shrink-0 items-center"
									:triggers="['hover', 'focus']"
									:popper-triggers="['hover', 'focus']"
									popper-class="v-popper--interactive export-selected-files-popper"
									placement="top"
									:delay="{ show: 200, hide: 100 }"
									no-auto-focus
								>
									<button
										type="button"
										class="inline-flex cursor-help items-center border-0 border-b border-dashed border-secondary bg-transparent p-0 text-sm font-semibold leading-none text-contrast"
										:aria-label="
											formatMessage(messages.selectedFilesTooltipTitle, {
												count: selectedFileCount,
											})
										"
									>
										{{ selectedFileCount }}
									</button>
									<template #popper>
										<div class="grid max-w-[34rem]">
											<div
												class="flex max-h-64 min-w-0 flex-col gap-2 overflow-y-auto overscroll-contain pr-2"
											>
												<span
													v-for="path in selectedFileSummary"
													:key="path"
													class="truncate whitespace-nowrap text-sm font-medium text-secondary"
												>
													{{ path }}
												</span>
											</div>
										</div>
									</template>
								</Tooltip>
								{{ selectedFileCountLabel }}
							</span>
						</template>
					</FileTreeSelect>
				</Collapsible>
			</div>
		</div>
		<template #actions>
			<div class="flex items-center justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="exportModal.hide">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button @click="exportPack">
						<PackageIcon />
						{{ formatMessage(messages.exportButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>
