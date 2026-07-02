<template>
	<div class="grid w-full grid-cols-1 gap-2 sm:grid-cols-2">
		<Combobox
			:model-value="currentGameVersion || undefined"
			class="w-full"
			:options="gameVersionOptions"
			:placeholder="formatMessage(messages.selectGameVersion)"
			:searchable="project.game_versions.length > 4"
			search-autocomplete="off"
			:search-placeholder="formatMessage(messages.searchGameVersions)"
			:no-options-message="formatMessage(messages.noGameVersionsFound)"
			trigger-class="!rounded-xl !bg-button-bg !px-3 !py-2"
			dropdown-class="!rounded-xl"
			@update:model-value="emit('selectGameVersion', $event)"
			@search-input="emit('update:versionFilter', $event)"
			@close="emit('update:versionFilter', '')"
		>
			<template #option="{ item, isSelected }">
				<div
					v-tooltip="
						!possibleGameVersions.includes(item.value)
							? formatMessage(messages.gameVersionUnsupportedTooltip, {
									title: project.title,
									gameVersion: item.value,
									platform: currentPlatformText,
								})
							: null
					"
					class="flex w-full items-center justify-between gap-2"
					:class="{
						'text-brand-red opacity-40': !possibleGameVersions.includes(item.value),
						'text-green': isSelected,
						'text-primary': possibleGameVersions.includes(item.value) && !isSelected,
					}"
				>
					<span class="min-w-0 truncate font-semibold leading-tight">{{ item.label }}</span>
				</div>
			</template>
			<template #dropdown-footer>
				<div v-if="showVersionsCheckbox" class="border-0 border-t border-solid border-surface-5 p-3">
					<Checkbox
						v-model="showAllVersionsModel"
						:label="formatMessage(messages.showAllVersions)"
						:disabled="!!versionFilter"
					/>
				</div>
			</template>
		</Combobox>
		<Combobox
			v-if="project.project_type !== 'resourcepack'"
			:model-value="currentPlatform || undefined"
			class="w-full"
			:options="platformOptions"
			:placeholder="formatMessage(messages.selectPlatform)"
			trigger-class="!rounded-xl !bg-button-bg !px-3 !py-2"
			dropdown-class="!rounded-xl"
			@update:model-value="emit('selectPlatform', $event)"
		>
			<template #option="{ item, isSelected }">
				<div
					v-tooltip="
						!possiblePlatforms.includes(item.value)
							? formatMessage(messages.platformUnsupportedTooltip, {
									title: project.title,
									platform: item.label,
									gameVersion: currentGameVersion,
								})
							: null
					"
					class="flex w-full items-center justify-between gap-2"
					:class="{
						'text-brand-red opacity-40': !possiblePlatforms.includes(item.value),
						'text-green': isSelected,
						'text-primary': possiblePlatforms.includes(item.value) && !isSelected,
					}"
				>
					<span class="min-w-0 truncate font-semibold leading-tight">{{ item.label }}</span>
				</div>
			</template>
		</Combobox>
	</div>

	<div
		v-if="selectedVersion"
		class="grid grid-cols-[minmax(0,1fr)_min-content] items-center gap-3 rounded-2xl bg-bg px-3 py-3"
	>
		<div class="flex min-w-0 flex-col gap-1">
			<div class="flex min-w-0 items-center gap-2">
				<span class="truncate font-bold text-contrast">
					{{ selectedVersion.version_number }}
				</span>
				<VersionChannelTag :channel="selectedVersion.version_type" class="!py-0.5" />
			</div>
			<p class="m-0 truncate text-sm text-secondary">
				{{ selectedVersion.name }}
			</p>
		</div>
		<ButtonStyled v-if="selectedPrimaryFile" color="brand" circular>
			<a
				:href="selectedPrimaryFileDownloadUrl"
				:download="selectedPrimaryFile.filename"
				:aria-label="
					formatMessage(messages.downloadVersion, {
						version: selectedVersion.version_number,
					})
				"
				v-tooltip="'Download'"
				@click="emit('download')"
			>
				<DownloadIcon aria-hidden="true" />
			</a>
		</ButtonStyled>
	</div>
</template>

<script setup>
import { DownloadIcon } from '@modrinth/assets'
import { ButtonStyled, Checkbox, Combobox, defineMessages, useVIntl } from '@modrinth/ui'
import VersionChannelTag from '@modrinth/ui/src/components/version/VersionChannelTag.vue'
import { computed } from 'vue'

defineOptions({
	name: 'DownloadProject',
})

const props = defineProps({
	project: {
		type: Object,
		required: true,
	},
	gameVersionOptions: {
		type: Array,
		default: () => [],
	},
	currentGameVersion: {
		type: [String, Boolean],
		default: null,
	},
	possibleGameVersions: {
		type: Array,
		default: () => [],
	},
	currentPlatformText: {
		type: String,
		default: null,
	},
	showVersionsCheckbox: {
		type: Boolean,
		default: false,
	},
	showAllVersions: {
		type: Boolean,
		default: false,
	},
	versionFilter: {
		type: String,
		default: '',
	},
	currentPlatform: {
		type: [String, Boolean],
		default: null,
	},
	platformOptions: {
		type: Array,
		default: () => [],
	},
	possiblePlatforms: {
		type: Array,
		default: () => [],
	},
	selectedVersion: {
		type: Object,
		default: null,
	},
	selectedPrimaryFile: {
		type: Object,
		default: null,
	},
	selectedPrimaryFileDownloadUrl: {
		type: String,
		default: null,
	},
})

const emit = defineEmits([
	'download',
	'selectGameVersion',
	'selectPlatform',
	'update:showAllVersions',
	'update:versionFilter',
])
const { formatMessage } = useVIntl()

const showAllVersionsModel = computed({
	get() {
		return props.showAllVersions
	},
	set(value) {
		emit('update:showAllVersions', value)
	},
})

const messages = defineMessages({
	gameVersionUnsupportedTooltip: {
		id: 'project.download.game-version-unsupported-tooltip',
		defaultMessage: '{title} does not support {gameVersion} for {platform}',
	},
	downloadVersion: {
		id: 'project.download.download-version',
		defaultMessage: 'Download {version}',
	},
	noGameVersionsFound: {
		id: 'project.download.no-game-versions-found',
		defaultMessage: 'No game versions found',
	},
	platformUnsupportedTooltip: {
		id: 'project.download.platform-unsupported-tooltip',
		defaultMessage: '{title} does not support {platform} for {gameVersion}',
	},
	searchGameVersions: {
		id: 'project.download.search-game-versions',
		defaultMessage: 'Select game version',
	},
	selectGameVersion: {
		id: 'project.download.select-game-version',
		defaultMessage: 'Select game version',
	},
	selectPlatform: {
		id: 'project.download.select-platform',
		defaultMessage: 'Select platform',
	},
	showAllVersions: {
		id: 'project.download.show-all-versions',
		defaultMessage: 'Show all versions',
	},
})
</script>
