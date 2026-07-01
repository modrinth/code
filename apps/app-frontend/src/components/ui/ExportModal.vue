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
import { ref } from 'vue'

import { PackageIcon } from '@/assets/icons'
import { export_instance_mrpack, get_pack_export_candidates } from '@/helpers/instance'

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
const loadedDirectories = ref(new Set())

async function initFiles() {
	const loadId = ++filesLoadId.value
	const exportCandidates = await get_pack_export_candidates(props.instance.id)
	if (loadId !== filesLoadId.value) return

	files.value = exportCandidates
	selectedFilePaths.value = files.value
		.filter((file) => !file.disabled && file.defaultSelected)
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
	loadedDirectories.value = new Set()
}

async function loadExportDirectory(path) {
	const normalizedPath = normalizeExportPath(path)
	if (!normalizedPath) return

	if (loadedDirectories.value.has(normalizedPath)) {
		replaceSelectedDirectoryWithLoadedChildren(
			normalizedPath,
			getLoadedDirectoryChildren(normalizedPath),
		)
		return
	}

	const loadId = filesLoadId.value
	loadedDirectories.value.add(normalizedPath)

	try {
		const childItems = await get_pack_export_candidates(props.instance.id, normalizedPath)
		if (loadId !== filesLoadId.value) return

		appendExportItems(childItems)
		replaceSelectedDirectoryWithLoadedChildren(normalizedPath, childItems)
	} catch {
		loadedDirectories.value.delete(normalizedPath)
	}
}

function appendExportItems(items) {
	const nextFiles = new Map(files.value.map((file) => [normalizeExportPath(file.path), file]))
	for (const item of items) {
		nextFiles.set(normalizeExportPath(item.path), item)
	}
	files.value = [...nextFiles.values()]
}

function getLoadedDirectoryChildren(path) {
	const normalizedPath = normalizeExportPath(path)
	const prefix = `${normalizedPath}/`

	return files.value.filter((item) => {
		const itemPath = normalizeExportPath(item.path)
		if (!itemPath.startsWith(prefix)) return false

		return itemPath.slice(prefix.length).split('/').filter(Boolean).length === 1
	})
}

function replaceSelectedDirectoryWithLoadedChildren(path, items) {
	const nextSelectedPaths = new Set(selectedFilePaths.value.map(normalizeExportPath))
	if (!nextSelectedPaths.delete(normalizeExportPath(path))) return

	for (const item of items) {
		if (item && !item.disabled) {
			nextSelectedPaths.add(normalizeExportPath(item.path))
		}
	}

	selectedFilePaths.value = [...nextSelectedPaths]
}

function normalizeExportPath(path) {
	return path.replaceAll('\\', '/').split('/').filter(Boolean).join('/')
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
