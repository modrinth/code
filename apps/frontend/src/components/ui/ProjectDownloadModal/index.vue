<template>
	<NewModal ref="modal" :on-show="onShow" :on-hide="onHide" width="544px">
		<template #title>
			<Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" />
			<div class="truncate text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.downloadTitle, { title: project.title }) }}
			</div>
		</template>
		<template #default>
			<div class="mx-auto flex w-full flex-col gap-4">
				<InstallWithModrinthApp :project="project" :tags="tags" />
				<DownloadProject
					:project="project"
					:versions="versions"
					:versions-loading="versionsLoading"
					:tags="tags"
					:download-reason="downloadReason"
					:initial-game-version="initialGameVersion"
					:initial-platform="initialPlatform"
					:reset-key="downloadProjectResetKey"
					@select-game-version="selectGameVersion"
					@select-platform="selectPlatform"
					@update:selection="projectDownloadSelection = $event"
					@download="onDownload"
				/>
				<div class="flex flex-col gap-4">
					<DownloadDependencies
						:project="project"
						:selected-version="selectedVersion"
						:current-game-version="currentGameVersion"
						:current-platform="currentPlatform"
						:download-reason="downloadReason"
						@download="onDownload"
					/>
					<div v-if="additionalFiles.length > 0" class="flex flex-col gap-2">
						<h3 class="m-0 text-sm font-bold text-contrast">
							{{ formatMessage(messages.additionalFilesTitle) }}
						</h3>
						<div class="flex flex-col gap-2">
							<a
								v-for="file in additionalFiles"
								:key="file.hashes?.sha1 ?? file.filename"
								:href="getDownloadUrl(file.url)"
								:download="file.filename"
								class="grid min-h-9 grid-cols-[minmax(0,1fr)_min-content] items-center gap-2 rounded-xl bg-button-bg px-3 py-2 text-primary no-underline"
								@click="onDownload"
							>
								<span class="flex min-w-0 items-center gap-2">
									<FileIcon aria-hidden="true" class="size-5 flex-shrink-0 text-secondary" />
									<span class="min-w-0 truncate font-semibold text-contrast">
										{{ file.filename }}
									</span>
									<span
										class="rounded-full bg-button-bgSelected px-2 py-0.5 text-xs text-secondary"
									>
										{{ fileTypeLabel(file.file_type) }}
									</span>
								</span>
								<DownloadIcon aria-hidden="true" class="size-5 text-secondary" />
							</a>
						</div>
					</div>
				</div>
				<ServersPromo
					v-if="flags.showProjectPageDownloadModalServersPromo"
					:link="`/hosting#plan`"
					@close="
						() => {
							flags.showProjectPageDownloadModalServersPromo = false
							saveFeatureFlags()
						}
					"
				/>
			</div>
		</template>
	</NewModal>
</template>

<script setup>
import { DownloadIcon, FileIcon } from '@modrinth/assets'
import {
	Avatar,
	defineMessages,
	NewModal,
	ServersPromo,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { navigateTo } from '#app'
import { saveFeatureFlags } from '~/composables/featureFlags.ts'

import DownloadDependencies from './DownloadDependencies.vue'
import DownloadProject from './DownloadProject.vue'
import InstallWithModrinthApp from './InstallWithModrinthApp.vue'

const props = defineProps({
	project: {
		type: Object,
		required: true,
	},
	versions: {
		type: Array,
		default: () => [],
	},
	versionsLoading: {
		type: Boolean,
		default: false,
	},
	tags: {
		type: Object,
		required: true,
	},
	downloadReason: {
		type: String,
		default: 'standalone',
	},
	loadVersions: {
		type: Function,
		required: true,
	},
})

const emit = defineEmits(['download'])

const route = useRoute()
const flags = useFeatureFlags()
const { createProjectDownloadUrl } = useCdnDownloadContext()
const { formatMessage } = useVIntl()
const debug = useDebugLogger('DownloadModal')

const modal = ref()
const modalOpen = ref(false)
const downloadProjectResetKey = ref(0)
const projectDownloadSelection = ref({
	currentGameVersion: null,
	currentPlatform: null,
	selectedVersion: null,
	selectedPrimaryFile: null,
})

const { version, loader } = route.query
const initialGameVersion = ref(
	typeof version === 'string' && props.project.game_versions.includes(version) ? version : null,
)
const initialPlatform = ref(
	typeof loader === 'string' && props.project.loaders.includes(loader) ? loader : null,
)

const currentGameVersion = computed(() => projectDownloadSelection.value.currentGameVersion)
const currentPlatform = computed(() => projectDownloadSelection.value.currentPlatform)
const selectedVersion = computed(() => projectDownloadSelection.value.selectedVersion)
const selectedPrimaryFile = computed(() => projectDownloadSelection.value.selectedPrimaryFile)

const additionalFiles = computed(() => {
	if (!selectedVersion.value || !selectedPrimaryFile.value) return []
	return selectedVersion.value.files.filter((file) => file !== selectedPrimaryFile.value)
})

const messages = defineMessages({
	downloadTitle: {
		id: 'project.download.title',
		defaultMessage: 'Download {title}',
	},
	additionalFilesTitle: {
		id: 'project.download.additional-files-title',
		defaultMessage: 'Additional files',
	},
})

function fileTypeLabel(type) {
	return (
		{
			'required-resource-pack': 'Resourcepack',
			'optional-resource-pack': 'Resourcepack',
			unknown: 'File',
		}[type] || 'File'
	)
}

function getDownloadUrl(url) {
	return createProjectDownloadUrl(url, {
		reason: props.downloadReason,
		gameVersion: currentGameVersion.value ?? undefined,
		loader: currentPlatform.value ?? undefined,
	})
}

function updateDownloadQuery({ gameVersion, platform }) {
	navigateTo(
		{
			query: {
				...route.query,
				...(gameVersion && {
					version: gameVersion,
				}),
				...(platform && {
					loader: platform,
				}),
			},
			hash: route.hash,
		},
		{ replace: true },
	)
}

function selectGameVersion(gameVersion) {
	updateDownloadQuery({
		gameVersion,
		platform: currentPlatform.value,
	})
}

function selectPlatform(platform) {
	updateDownloadQuery({
		gameVersion: currentGameVersion.value,
		platform,
	})
}

function onShow() {
	modalOpen.value = true
	debug('on-show fired')
	props.loadVersions()
	navigateTo({ query: route.query, hash: '#download' }, { replace: true })
}

function onHide() {
	modalOpen.value = false
	navigateTo({ query: route.query, hash: '' }, { replace: true })
}

function show(event) {
	if (!modal.value || modalOpen.value) return
	modalOpen.value = true
	modal.value.show(event)
}

function hide(event) {
	if (!modal.value || !modalOpen.value) return
	modal.value?.hide(event)
	downloadProjectResetKey.value += 1
}

function onDownload() {
	emit('download')
}

function openFromHash() {
	if (!modal.value || modalOpen.value || route.hash !== '#download') return

	debug('hash #download watch fired, opening modal')
	show()
}

if (route.hash === '#download' || version !== undefined || loader !== undefined) {
	debug('eager loadVersions from setup', { hash: route.hash, version, loader })
	props.loadVersions()
}

watch(modal, openFromHash)
watch(() => route.hash, openFromHash)

defineExpose({ show, hide })
</script>
