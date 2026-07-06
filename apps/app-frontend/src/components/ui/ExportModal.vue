<script setup>
import { XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	FileTreeSelect,
	injectNotificationManager,
	NewModal,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { save } from '@tauri-apps/plugin-dialog'
import { readDir, stat } from '@tauri-apps/plugin-fs'
import { ref } from 'vue'

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
	exportButton: { id: 'app.export-modal.export-button', defaultMessage: 'Export' },
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
		void initFiles().catch(handleError)
	},
})

const exportModal = ref(null)
const nameInput = ref(props.instance.name)
const exportDescription = ref('')
const versionInput = ref('1.0.0')
const files = ref([])
const selectedFilePaths = ref([])
const fileTreeKey = ref(0)
const filesLoadId = ref(0)
const instanceRoot = ref('')
const loadedDirectories = ref(new Set())

async function initFiles() {
	const loadId = ++filesLoadId.value
	const [filePaths, root] = await Promise.all([
		get_pack_export_candidates(props.instance.id),
		get_full_path(props.instance.id),
	])
	if (loadId !== filesLoadId.value) return

	instanceRoot.value = root
	const exportCandidates = await Promise.all(
		filePaths.map((path) => buildExportCandidateItem(root, path)),
	)
	if (loadId !== filesLoadId.value) return

	files.value = exportCandidates
	selectedFilePaths.value = files.value
		.filter((file) => !file.disabled && isDefaultSelectedExportCandidate(file.path))
		.map((file) => file.path)
}

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
	fileTreeKey.value += 1
	instanceRoot.value = ''
	loadedDirectories.value = new Set()
}

async function loadExportDirectory(path) {
	if (!path || !instanceRoot.value || loadedDirectories.value.has(path)) return

	const loadId = filesLoadId.value
	loadedDirectories.value.add(path)

	try {
		const entries = await readDir(`${instanceRoot.value}/${path}`)
		const childItems = await Promise.all(
			entries.map((entry) => buildExportDirectoryChildItem(instanceRoot.value, path, entry)),
		)
		if (loadId !== filesLoadId.value) return

		appendExportItems(childItems)
	} catch {
		loadedDirectories.value.delete(path)
	}
}

async function buildExportCandidateItem(instanceRoot, path) {
	try {
		const entries = await readDir(`${instanceRoot}/${path}`)
		const metadata = await getExportCandidateMetadata(instanceRoot, path)
		return {
			path,
			type: 'directory',
			disabled: isExportCandidateDisabled(path),
			modified: metadata.modified,
			count: entries.length,
		}
	} catch {
		return buildExportFileItem(instanceRoot, path)
	}
}

async function buildExportDirectoryChildItem(instanceRoot, parentPath, entry) {
	const path = `${parentPath}/${entry.name}`
	if (entry.isDirectory) {
		const metadata = await getExportCandidateMetadata(instanceRoot, path)
		return {
			path,
			type: 'directory',
			disabled: isExportCandidateDisabled(path),
			modified: metadata.modified,
		}
	}

	return buildExportFileItem(instanceRoot, path)
}

async function buildExportFileItem(instanceRoot, path) {
	const metadata = await getExportCandidateMetadata(instanceRoot, path)
	return {
		path,
		type: 'file',
		disabled: isExportCandidateDisabled(path),
		size: metadata.size,
		modified: metadata.modified,
	}
}

function appendExportItems(items) {
	const nextFiles = new Map(files.value.map((file) => [normalizeExportPath(file.path), file]))
	for (const item of items) {
		nextFiles.set(normalizeExportPath(item.path), item)
	}
	files.value = [...nextFiles.values()]
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

function normalizeExportPath(path) {
	return path.replaceAll('\\', '/').split('/').filter(Boolean).join('/')
}

function isDefaultSelectedExportCandidate(path) {
	return (
		path.startsWith('mods') ||
		path.startsWith('datapacks') ||
		path.startsWith('resourcepacks') ||
		path.startsWith('shaderpacks') ||
		path.startsWith('config')
	)
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
				<StyledInput
					v-model="exportDescription"
					multiline
					:placeholder="formatMessage(messages.descriptionPlaceholder)"
					wrapper-class="w-full"
				/>
			</div>
			<FileTreeSelect
				:key="fileTreeKey"
				v-model="selectedFilePaths"
				class="min-w-0"
				:items="files"
				@navigate="loadExportDirectory"
			/>
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
