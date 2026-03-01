<template>
	<NewModal ref="modal" no-padding scrollable max-width="560px" width="560px" :on-hide="handleHide">
		<template #title>
			<span class="text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.header) }}
			</span>
		</template>

		<div class="flex flex-col gap-2.5 p-6">
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.instanceType) }}
			</span>
			<Chips v-model="tab" :items="tabs" :format-label="formatTabLabel" :never-empty="true" />
		</div>

		<div class="h-px bg-divider" />

		<!-- Existing instance tab -->
		<div
			v-if="tab === 'existing'"
			class="flex flex-col gap-3 bg-surface-2 px-6 py-4"
			style="min-height: 300px; max-height: 400px; overflow-y: auto"
		>
			<div class="flex items-start gap-3">
				<StyledInput
					v-model="searchFilter"
					:icon="SearchIcon"
					:placeholder="formatMessage(messages.searchPlaceholder)"
					class="flex-1"
				/>
				<ButtonStyled type="outlined" circular>
					<button
						v-tooltip="`${hideIncompatible ? 'Show' : 'Hide'} incompatible`"
						class="!border-surface-4 !border"
						@click="hideIncompatible = !hideIncompatible"
					>
						<EyeIcon v-if="hideIncompatible" />
						<EyeOffIcon v-else />
					</button>
				</ButtonStyled>
			</div>

			<div v-if="loading" class="flex items-center justify-center py-12">
				<LoadingIndicator />
			</div>
			<div
				v-else-if="filteredInstances.length === 0"
				class="flex items-center justify-center py-12 text-secondary"
			>
				{{ formatMessage(messages.noInstances) }}
			</div>
			<div v-else class="flex flex-col gap-1">
				<div
					v-for="inst in filteredInstances"
					:key="inst.id"
					class="flex h-10 items-center justify-between rounded-lg px-2"
					:class="
						!inst.compatible ? 'opacity-40' : inst.installed ? 'opacity-60' : 'hover:bg-surface-3'
					"
				>
					<div
						v-tooltip="
							!inst.compatible ? 'This instance is not compatible with this project' : undefined
						"
						class="flex items-center gap-2 overflow-hidden"
					>
						<Avatar :src="inst.iconUrl ?? undefined" size="20px" rounded="md" />
						<span class="truncate font-semibold text-contrast">{{ inst.name }}</span>
					</div>
					<ButtonStyled v-if="inst.installed" :disabled="true">
						<button>
							<CheckIcon />
							{{ formatMessage(messages.installedBadge) }}
						</button>
					</ButtonStyled>
					<ButtonStyled v-else-if="inst.compatible" :disabled="inst.installing">
						<button @click="emit('install', inst)">
							{{
								inst.installing
									? formatMessage(messages.installingLabel)
									: formatMessage(messages.installButton)
							}}
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>

		<!-- New instance tab -->
		<div v-else class="flex flex-col gap-6 p-6">
			<div class="flex items-center gap-4">
				<Avatar :src="iconPreviewUrl ?? undefined" size="5rem" rounded="2xl" />
				<div class="flex flex-col gap-2">
					<ButtonStyled type="outlined">
						<button class="!border-surface-4 !border" @click="selectIcon">
							<UploadIcon />
							{{ formatMessage(messages.selectIcon) }}
						</button>
					</ButtonStyled>
					<ButtonStyled type="outlined">
						<button
							class="!border-surface-4 !border"
							:disabled="!iconPreviewUrl"
							@click="removeIcon"
						>
							<XIcon />
							{{ formatMessage(messages.removeIcon) }}
						</button>
					</ButtonStyled>
				</div>
			</div>

			<div class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.nameLabel) }}
				</span>
				<StyledInput
					v-model="instanceName"
					:placeholder="formatMessage(messages.namePlaceholder)"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.loaderLabel) }}
				</span>
				<Chips
					v-model="selectedLoader"
					:items="compatibleLoaders"
					:format-label="formatLoaderLabel"
					:never-empty="true"
				/>
			</div>

			<div class="flex flex-col gap-2.5">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.gameVersionLabel) }}
				</span>
				<Combobox
					v-model="selectedGameVersion"
					:options="gameVersionOptions"
					searchable
					sync-with-selection
					:placeholder="formatMessage(messages.gameVersionPlaceholder)"
				>
					<template v-if="hasReleaseData" #dropdown-footer>
						<button
							class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
							@mousedown.prevent
							@click="showSnapshots = !showSnapshots"
						>
							<EyeOffIcon v-if="showSnapshots" class="size-4" />
							<EyeIcon v-else class="size-4" />
							{{
								showSnapshots
									? formatMessage(messages.hideSnapshots)
									: formatMessage(messages.showAllVersions)
							}}
						</button>
					</template>
				</Combobox>
			</div>
		</div>

		<template #actions>
			<div v-if="tab === 'existing'" class="flex items-center justify-between pt-5 pb-1 px-4">
				<div class="flex items-center gap-1.5">
					<BoxIcon class="size-5" />
					<span>
						{{ formatMessage(messages.compatibleCount, { count: compatibleCount }) }}
					</span>
				</div>
				<ButtonStyled type="outlined">
					<button class="!border-surface-4 !border" @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
			</div>

			<div v-else class="flex items-center justify-end gap-2">
				<ButtonStyled type="outlined">
					<button class="!border-surface-4 !border" @click="modal?.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!instanceName" @click="handleCreateAndInstall">
						<DownloadIcon />
						{{ formatMessage(messages.installButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	CheckIcon,
	DownloadIcon,
	EyeIcon,
	EyeOffIcon,
	SearchIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import { defineMessages, useVIntl } from '../../../composables/i18n'
import { injectFilePicker } from '../../../providers'
import { commonMessages } from '../../../utils/common-messages'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import Chips from '../../base/Chips.vue'
import Combobox, { type ComboboxOption } from '../../base/Combobox.vue'
import LoadingIndicator from '../../base/LoadingIndicator.vue'
import StyledInput from '../../base/StyledInput.vue'
import NewModal from '../../modal/NewModal.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'instances.content-install.header',
		defaultMessage: 'Install project',
	},
	instanceType: {
		id: 'instances.content-install.instance-type',
		defaultMessage: 'Instance type',
	},
	existingTab: {
		id: 'instances.content-install.existing-tab',
		defaultMessage: 'Existing instance',
	},
	newTab: {
		id: 'instances.content-install.new-tab',
		defaultMessage: 'New instance',
	},
	searchPlaceholder: {
		id: 'instances.content-install.search-placeholder',
		defaultMessage: 'Search instance',
	},
	installedBadge: {
		id: 'instances.content-install.installed-badge',
		defaultMessage: 'Installed',
	},
	installingLabel: {
		id: 'instances.content-install.installing-label',
		defaultMessage: 'Installing...',
	},
	installButton: {
		id: 'instances.content-install.install-button',
		defaultMessage: 'Install',
	},
	selectIcon: {
		id: 'instances.content-install.select-icon',
		defaultMessage: 'Select icon',
	},
	removeIcon: {
		id: 'instances.content-install.remove-icon',
		defaultMessage: 'Remove icon',
	},
	nameLabel: {
		id: 'instances.content-install.name-label',
		defaultMessage: 'Name',
	},
	namePlaceholder: {
		id: 'instances.content-install.name-placeholder',
		defaultMessage: 'Enter instance name',
	},
	loaderLabel: {
		id: 'instances.content-install.loader-label',
		defaultMessage: 'Loader',
	},
	gameVersionLabel: {
		id: 'instances.content-install.game-version-label',
		defaultMessage: 'Game version',
	},
	gameVersionPlaceholder: {
		id: 'instances.content-install.game-version-placeholder',
		defaultMessage: 'Select game version',
	},
	compatibleCount: {
		id: 'instances.content-install.compatible-count',
		defaultMessage: '{count} compatible {count, plural, one {instance} other {instances}}',
	},
	noInstances: {
		id: 'instances.content-install.no-instances',
		defaultMessage: 'No compatible instances found',
	},
	showAllVersions: {
		id: 'instances.content-install.show-all-versions',
		defaultMessage: 'Show all versions',
	},
	hideSnapshots: {
		id: 'instances.content-install.hide-snapshots',
		defaultMessage: 'Hide snapshots',
	},
})

const loaderDisplayNames: Record<string, string> = {
	fabric: 'Fabric',
	quilt: 'Quilt',
	forge: 'Forge',
	neoforge: 'NeoForge',
	paper: 'Paper',
	purpur: 'Purpur',
	vanilla: 'Vanilla',
}

const formatLoaderLabel = (item: string) =>
	loaderDisplayNames[item] ?? item.charAt(0).toUpperCase() + item.slice(1)

export interface ContentInstallInstance {
	id: string
	name: string
	iconUrl?: string | null
	installed: boolean
	compatible: boolean
	installing?: boolean
}

const props = defineProps<{
	instances: ContentInstallInstance[]
	compatibleLoaders: string[]
	gameVersions: string[]
	releaseGameVersions?: Set<string>
	loading?: boolean
	defaultTab?: 'existing' | 'new'
	preferredLoader?: string | null
	preferredGameVersion?: string | null
}>()

const emit = defineEmits<{
	install: [instance: ContentInstallInstance]
	'create-and-install': [
		data: {
			name: string
			iconPath: string | null
			iconPreviewUrl: string | null
			loader: string
			gameVersion: string
		},
	]
	cancel: []
}>()

const modal = ref<InstanceType<typeof NewModal>>()

type Tab = 'existing' | 'new'
const tabs = computed<Tab[]>(() =>
	props.compatibleLoaders.length > 0 ? ['existing', 'new'] : ['existing'],
)
const tab = ref<Tab>('existing')

const tabLabels: Record<Tab, () => string> = {
	existing: () => formatMessage(messages.existingTab),
	new: () => formatMessage(messages.newTab),
}
const formatTabLabel = (item: Tab) => tabLabels[item]()

const searchFilter = ref('')
const hideIncompatible = ref(true)

const filteredInstances = computed(() => {
	let list = props.instances
	if (hideIncompatible.value) list = list.filter((i) => i.compatible)
	if (searchFilter.value) {
		const query = searchFilter.value.toLowerCase()
		list = list.filter((i) => i.name.toLowerCase().includes(query))
	}
	return list.slice().sort((a, b) => {
		const aScore = a.installed ? 1 : 0
		const bScore = b.installed ? 1 : 0
		if (aScore !== bScore) return aScore - bScore
		return a.name.localeCompare(b.name)
	})
})

const compatibleCount = computed(() => props.instances.filter((i) => i.compatible).length)

const instanceName = ref('')
const selectedLoader = ref<string | null>(null)
const selectedGameVersion = ref<string | null>(null)
const iconPath = ref<string | null>(null)
const iconPreviewUrl = ref<string | null>(null)
const showSnapshots = ref(false)

const hasReleaseData = computed(
	() => props.releaseGameVersions && props.releaseGameVersions.size > 0,
)

const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	const versions =
		showSnapshots.value || !hasReleaseData.value
			? props.gameVersions
			: props.gameVersions.filter((v) => props.releaseGameVersions!.has(v))
	return versions.map((v) => ({ value: v, label: v }))
})

const filePicker = injectFilePicker(null)

async function selectIcon() {
	if (!filePicker) return
	const picked = await filePicker.pickImage()
	if (picked) {
		iconPath.value = picked.path ?? null
		iconPreviewUrl.value = picked.previewUrl
	}
}

function removeIcon() {
	iconPath.value = null
	iconPreviewUrl.value = null
}

function resetState() {
	tab.value = props.defaultTab ?? 'existing'
	searchFilter.value = ''
	hideIncompatible.value = true
	instanceName.value = ''
	iconPath.value = null
	iconPreviewUrl.value = null
	selectedLoader.value = props.preferredLoader ?? props.compatibleLoaders[0] ?? null

	const preferred = props.preferredGameVersion
	const isSnapshot = preferred && hasReleaseData.value && !props.releaseGameVersions!.has(preferred)
	showSnapshots.value = !!isSnapshot

	const defaultVersion = hasReleaseData.value
		? (props.gameVersions.find((v) => props.releaseGameVersions!.has(v)) ??
			props.gameVersions[0] ??
			null)
		: (props.gameVersions[0] ?? null)
	selectedGameVersion.value = preferred ?? defaultVersion
}

function handleHide() {
	resetState()
	emit('cancel')
}

function show() {
	resetState()
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function handleCreateAndInstall() {
	if (!instanceName.value || !selectedLoader.value || !selectedGameVersion.value) return
	emit('create-and-install', {
		name: instanceName.value,
		iconPath: iconPath.value,
		iconPreviewUrl: iconPreviewUrl.value,
		loader: selectedLoader.value,
		gameVersion: selectedGameVersion.value,
	})
}

defineExpose({ show, hide })
</script>
