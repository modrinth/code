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
import { ref, shallowRef } from 'vue'

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
const files = shallowRef([])
const includedFilePaths = ref([])
const excludedFilePaths = ref([])
const fileTreeKey = ref(0)
const filesLoadId = ref(0)
const directoryEntries = new Map()
const currentDirectory = ref('')

async function initFiles() {
	const loadId = ++filesLoadId.value
	const exportCandidates = await get_pack_export_candidates(props.instance.id)
	if (loadId !== filesLoadId.value) return

	files.value = exportCandidates
	directoryEntries.set('', exportCandidates)
	currentDirectory.value = ''
	includedFilePaths.value = files.value
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
			includedFilePaths.value,
			excludedFilePaths.value,
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
	includedFilePaths.value = []
	excludedFilePaths.value = []
	fileTreeKey.value += 1
	directoryEntries.clear()
	currentDirectory.value = ''
}

async function loadExportDirectory(path) {
	const normalizedPath = normalizeExportPath(path)
	currentDirectory.value = normalizedPath

	const cachedEntries = directoryEntries.get(normalizedPath)
	if (cachedEntries) {
		files.value = cachedEntries
		return
	}

	const loadId = filesLoadId.value
	files.value = []

	try {
		const childItems = await get_pack_export_candidates(
			props.instance.id,
			normalizedPath || undefined,
		)
		if (loadId !== filesLoadId.value) return

		directoryEntries.set(normalizedPath, childItems)
		if (currentDirectory.value === normalizedPath) {
			files.value = childItems
		}
	} catch {
		if (currentDirectory.value === normalizedPath) files.value = []
	}
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
				v-model="includedFilePaths"
				v-model:excluded-paths="excludedFilePaths"
				class="min-w-0"
				:items="files"
				lazy
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
