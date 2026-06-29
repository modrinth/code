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
		initFiles()
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
