<template>
	<div class="flex flex-col gap-6">
		<div class="flex flex-col gap-4">
			<div class="flex flex-wrap items-start justify-between gap-3">
				<div class="flex min-w-0 flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.title) }}
					</span>
					<span class="text-primary">{{ formatMessage(messages.description) }}</span>
				</div>
			</div>

			<div class="flex items-start gap-2 text-primary">
				<InfoIcon class="mt-0.5 size-4 shrink-0" />
				<span>{{ formatMessage(messages.selectionDescription) }}</span>
			</div>

			<div v-if="configFilesQuery.isLoading.value" class="flex items-center gap-2 text-primary">
				<SpinnerIcon class="size-4 animate-spin" />
				{{ formatMessage(messages.loading) }}
			</div>
			<div v-else-if="configFilesQuery.isError.value" class="text-red">
				{{ formatMessage(messages.loadError) }}
			</div>
			<div v-else class="relative rounded-[20px]">
				<div
					ref="configFileTable"
					class="max-h-[292px] overflow-y-auto rounded-[20px]"
					@scroll="checkTableScrollState"
				>
					<FileTreeSelect
						v-model="selectedConfigFilePaths"
						:items="configFileItems"
						:show-size="false"
						:show-modified="false"
						@navigate="refreshTableScrollState"
					/>
				</div>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-2"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-2"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTableTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 rounded-t-[20px] bg-gradient-to-b from-bg-raised to-transparent"
					/>
				</Transition>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-2"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-2"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTableBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 rounded-b-[20px] bg-gradient-to-t from-bg-raised to-transparent"
					/>
				</Transition>
			</div>
		</div>

		<div>
			<SharedInstanceInstallationSettingsControls
				can-unpublish
				:busy="isBusy"
				:unpublishing="unpublishing"
				:unpublish="unpublishSharedInstance"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { InfoIcon, SpinnerIcon } from '@modrinth/assets'
import {
	defineMessages,
	FileTreeSelect,
	type FileTreeSelectItem,
	injectNotificationManager,
	useScrollIndicator,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import SharedInstanceInstallationSettingsControls from '@/components/ui/shared-instances/SharedInstanceInstallationSettingsControls.vue'
import {
	get_shared_instance_config_files,
	set_shared_instance_config_files,
	unpublish_shared_instance,
} from '@/helpers/instance'
import { injectInstanceSettings } from '@/providers/instance-settings'

const { instance, offline, onUnlinked } = injectInstanceSettings()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const unpublishing = ref(false)
const saving = ref(false)
const configFileTable = ref<HTMLElement | null>(null)
const selectionLoaded = ref(false)
const savedConfigFilePaths = ref<string[]>([])
const selectedConfigFilePaths = ref<string[]>([])
let saveTimeout: ReturnType<typeof setTimeout> | undefined
const {
	showTopFade: showTableTopFade,
	showBottomFade: showTableBottomFade,
	checkScrollState: checkTableScrollState,
	forceCheck: forceCheckTableScroll,
} = useScrollIndicator(configFileTable)

const configFilesQuery = useQuery({
	queryKey: computed(() => ['sharedInstanceConfigFiles', instance.value.id]),
	queryFn: () => get_shared_instance_config_files(instance.value.id),
	enabled: computed(() => instance.value.shared_instance?.role === 'owner' && !offline),
})
const configFileItems = computed<FileTreeSelectItem[]>(() =>
	(configFilesQuery.data.value?.files ?? []).map((path) => ({ path, type: 'file' })),
)
const hasSelectionChanges = computed(
	() => !sameConfigFilePaths(selectedConfigFilePaths.value, savedConfigFilePaths.value),
)
const isBusy = computed(
	() =>
		instance.value.install_stage !== 'installed' ||
		unpublishing.value ||
		saving.value ||
		hasSelectionChanges.value ||
		!!offline,
)

watch(
	() => configFilesQuery.data.value,
	(data) => {
		if (!data) return
		savedConfigFilePaths.value = [...data.selected]
		if (!selectionLoaded.value) {
			selectedConfigFilePaths.value = [...data.selected]
			selectionLoaded.value = true
		}
		void nextTick(() => forceCheckTableScroll())
	},
	{ immediate: true },
)

watch(selectedConfigFilePaths, () => {
	if (!selectionLoaded.value || !hasSelectionChanges.value || offline) return
	clearTimeout(saveTimeout)
	saveTimeout = setTimeout(() => {
		saveTimeout = undefined
		void saveSelection()
	}, 300)
})

onBeforeUnmount(() => {
	clearTimeout(saveTimeout)
	if (hasSelectionChanges.value && !offline) {
		void saveSelection()
	}
})

function refreshTableScrollState() {
	void nextTick(() => forceCheckTableScroll())
}

async function saveSelection() {
	if (saving.value || !selectionLoaded.value || !hasSelectionChanges.value || offline) return
	saving.value = true
	try {
		while (hasSelectionChanges.value) {
			const requested = [...selectedConfigFilePaths.value]
			const result = await set_shared_instance_config_files(instance.value.id, requested)
			queryClient.setQueryData(['sharedInstanceConfigFiles', instance.value.id], result)
			savedConfigFilePaths.value = [...result.selected]
			if (sameConfigFilePaths(selectedConfigFilePaths.value, requested)) {
				selectedConfigFilePaths.value = [...result.selected]
			}
		}
	} catch (error) {
		selectedConfigFilePaths.value = [...savedConfigFilePaths.value]
		handleError(error)
	} finally {
		saving.value = false
	}
}

function sameConfigFilePaths(left: string[], right: string[]) {
	if (left.length !== right.length) return false
	const sortedLeft = [...left].sort()
	const sortedRight = [...right].sort()
	return sortedLeft.every((path, index) => path === sortedRight[index])
}

async function unpublishSharedInstance() {
	unpublishing.value = true
	try {
		await unpublish_shared_instance(instance.value.id)
		await queryClient.invalidateQueries({ queryKey: ['sharedInstanceUsers', instance.value.id] })
		await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', instance.value.id] })
		onUnlinked()
	} catch (error) {
		handleError(error)
	} finally {
		unpublishing.value = false
	}
}

const messages = defineMessages({
	title: {
		id: 'instance.settings.tabs.sharing.config-files.title',
		defaultMessage: 'Config file sharing',
	},
	description: {
		id: 'instance.settings.tabs.sharing.config-files.description',
		defaultMessage: 'Choose which files are kept in sync for everyone using this shared instance.',
	},
	selectionDescription: {
		id: 'instance.settings.tabs.sharing.config-files.selection-description',
		defaultMessage: 'New config files will not be shared unless you select them here.',
	},
	loading: {
		id: 'instance.settings.tabs.sharing.config-files.loading',
		defaultMessage: 'Loading config files...',
	},
	loadError: {
		id: 'instance.settings.tabs.sharing.config-files.load-error',
		defaultMessage: 'Failed to load config files.',
	},
})
</script>
