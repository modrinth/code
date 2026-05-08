<script setup>
import { WrenchIcon, XIcon } from '@modrinth/assets'
import {
	Accordion,
	ButtonStyled,
	Checkbox,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { save } from '@tauri-apps/plugin-dialog'
import { ref } from 'vue'

import { PackageIcon, VersionIcon } from '@/assets/icons'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { export_profile_mrpack, get_pack_export_candidates } from '@/helpers/profile.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: { id: 'app.export-modal.header', defaultMessage: 'Export modpack' },
	modpackNameLabel: { id: 'app.export-modal.modpack-name-label', defaultMessage: 'Modpack Name' },
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
		defaultMessage: 'Configure which files are included in this export',
	},
	exportButton: { id: 'app.export-modal.export-button', defaultMessage: 'Export' },
	includeFile: {
		id: 'app.export-modal.include-file-accessibility-label',
		defaultMessage: 'Include "{file}"?',
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
		exportModal.value.show()
		initFiles()
	},
})

const exportModal = ref(null)
const nameInput = ref(props.instance.name)
const exportDescription = ref('')
const versionInput = ref('1.0.0')
const files = ref([])
const folders = ref([])

const initFiles = async () => {
	const newFolders = new Map()
	const sep = '/'
	files.value = []
	await get_pack_export_candidates(props.instance.path).then((filePaths) =>
		filePaths
			.map((folder) => ({
				path: folder,
				name: folder.split(sep).pop(),
				selected:
					folder.startsWith('mods') ||
					folder.startsWith('datapacks') ||
					folder.startsWith('resourcepacks') ||
					folder.startsWith('shaderpacks') ||
					folder.startsWith('config'),
				disabled:
					folder === 'profile.json' ||
					folder.startsWith('modrinth_logs') ||
					folder.startsWith('.fabric'),
			}))
			.filter(
				(pathData) =>
					!pathData.path.includes('.DS_Store') &&
					pathData.path !== 'mods/.connector' &&
					!pathData.path.startsWith('mods/.connector/'),
			)
			.forEach((pathData) => {
				const parent = pathData.path.split(sep).slice(0, -1).join(sep)
				if (parent !== '') {
					if (newFolders.has(parent)) {
						newFolders.get(parent).push(pathData)
					} else {
						newFolders.set(parent, [pathData])
					}
				} else {
					files.value.push(pathData)
				}
			}),
	)
	folders.value = [...newFolders.entries()].map(([name, value]) => [
		{
			name,
			showingMore: false,
		},
		value,
	])
}

await initFiles()

const exportPack = async () => {
	const filesToExport = files.value.filter((file) => file.selected).map((file) => file.path)
	folders.value.forEach((args) => {
		args[1].forEach((child) => {
			if (child.selected) {
				filesToExport.push(child.path)
			}
		})
	})
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
		export_profile_mrpack(
			props.instance.path,
			outputPath,
			filesToExport,
			versionInput.value,
			exportDescription.value,
			nameInput.value,
		).catch((err) => handleError(err))
		exportModal.value.hide()
	}
}
</script>

<template>
	<ModalWrapper ref="exportModal" :header="formatMessage(messages.header)">
		<div class="flex flex-col gap-4 w-[40rem]">
			<div class="grid grid-cols-2 gap-4">
				<div class="labeled_input">
					<p>{{ formatMessage(messages.modpackNameLabel) }}</p>
					<StyledInput
						v-model="nameInput"
						:icon="PackageIcon"
						type="text"
						:placeholder="formatMessage(messages.modpackNamePlaceholder)"
						clearable
					/>
				</div>
				<div class="labeled_input">
					<p>{{ formatMessage(messages.versionNumberLabel) }}</p>
					<StyledInput
						v-model="versionInput"
						:icon="VersionIcon"
						type="text"
						:placeholder="formatMessage(messages.versionNumberPlaceholder)"
						clearable
					/>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<p class="m-0">{{ formatMessage(commonMessages.descriptionLabel) }}</p>
				<StyledInput
					v-model="exportDescription"
					multiline
					:placeholder="formatMessage(messages.descriptionPlaceholder)"
				/>
			</div>
			<Accordion
				class="w-full bg-surface-4 border border-solid border-surface-5 rounded-2xl overflow-clip"
				button-class="p-4 w-full border-b border-solid border-b-surface-5 bg-surface-2 -mb-px hover:brightness-[--hover-brightness] group"
			>
				<template #title>
					<span class="flex items-center gap-3 text-contrast group-active:scale-[0.98]">
						<WrenchIcon aria-hidden="true" class="size-5 text-secondary" />
						Configure which files are included in this export
					</span>
				</template>
				<div class="flex flex-col [&>*:nth-child(even)]:bg-surface-3">
					<div v-for="[path, children] in folders" :key="path.name" class="flex flex-col">
						<Accordion
							class="flex flex-col"
							button-class="flex gap-3 pr-4 hover:bg-surface-5 group"
						>
							<template #title>
								<Checkbox
									:model-value="children.every((child) => child.selected)"
									:indeterminate="
										!children.every((child) => child.selected) &&
										children.some((child) => child.selected)
									"
									:description="formatMessage(messages.includeFile, { file: path.name })"
									class="pl-4 py-2"
									:disabled="children.every((x) => x.disabled)"
									@update:model-value="
										(newValue) => children.forEach((child) => (child.selected = newValue))
									"
									@click.stop
								/>
								<span class="ml-2 group-active:scale-95">{{ path.name }}/</span>
							</template>
							<div v-for="child in children" :key="child.path">
								<Checkbox
									v-model="child.selected"
									:label="child.name"
									class="w-full px-8 py-2 hover:bg-surface-4 text-primary"
									:disabled="child.disabled"
								/>
							</div>
						</Accordion>
					</div>
					<Checkbox
						v-for="file in files"
						:key="file.path"
						v-model="file.selected"
						:label="file.name"
						:disabled="file.disabled"
						class="w-full px-4 py-2 hover:bg-surface-4 text-primary"
					/>
				</div>
			</Accordion>
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
		</div>
	</ModalWrapper>
</template>
