<template>
	<NewModal ref="modal" :on-show="onShow" :on-hide="onHide">
		<template #title>
			<Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" />
			<div class="truncate text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.downloadTitle, { title: project.title }) }}
			</div>
		</template>
		<template #default>
			<div class="mx-auto flex max-w-[44rem] flex-col gap-4 md:w-[30rem]">
				<div
					v-if="
						project.project_type !== 'plugin' ||
						project.loaders.some((x) => !tags.loaderData.allPluginLoaders.includes(x))
					"
					class="modrinth-app-section contents"
				>
					<div class="mx-auto flex w-fit flex-col">
						<ButtonStyled color="brand">
							<a class="w-fit" :href="`modrinth://mod/${project.slug}`" @click="installWithApp">
								<ModrinthIcon aria-hidden="true" />
								{{ formatMessage(messages.installWithModrinthApp) }}
								<ExternalIcon aria-hidden="true" />
							</a>
						</ButtonStyled>
						<Accordion ref="getModrinthAppAccordion">
							<nuxt-link class="mt-2 flex justify-center text-brand-blue hover:underline" to="/app">
								{{ formatMessage(messages.dontHaveModrinthApp) }}
							</nuxt-link>
						</Accordion>
					</div>

					<div class="flex items-center gap-4 px-4">
						<div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
						<span class="flex-shrink-0 text-sm font-semibold text-secondary">
							{{ formatMessage(commonMessages.orLabel) }}
						</span>
						<div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
					</div>
				</div>

				<div class="mx-auto flex w-fit flex-col gap-2">
					<ButtonStyled v-if="project.game_versions.length === 1">
						<div class="disabled button-like">
							<GameIcon aria-hidden="true" />
							{{
								currentGameVersion
									? formatMessage(messages.gameVersionLabel, { version: currentGameVersion })
									: formatMessage(messages.gameVersionError)
							}}
							<InfoIcon
								v-tooltip="
									formatMessage(messages.gameVersionTooltip, {
										title: project.title,
										version: currentGameVersion,
									})
								"
								class="ml-auto size-5"
							/>
						</div>
					</ButtonStyled>
					<Accordion
						v-else
						ref="gameVersionAccordion"
						class="accordion-with-bg"
						@on-open="
							() => {
								platformAccordion?.close()
							}
						"
					>
						<template #title>
							<GameIcon aria-hidden="true" />
							{{
								currentGameVersion
									? formatMessage(messages.gameVersionLabel, { version: currentGameVersion })
									: formatMessage(messages.selectGameVersion)
							}}
						</template>
						<label for="game-versions-filtering" hidden>{{
							formatMessage(messages.searchGameVersionsLabel)
						}}</label>
						<StyledInput
							id="game-versions-filtering"
							ref="gameVersionFilterInput"
							v-model="versionFilter"
							type="search"
							autocomplete="off"
							:icon="SearchIcon"
							:placeholder="formatMessage(messages.searchGameVersions)"
							wrapper-class="mb-2 w-full"
						/>
						<ScrollablePanel :class="project.game_versions.length > 4 ? 'h-[15rem]' : ''">
							<ButtonStyled
								v-for="gameVersion in filteredGameVersions"
								:key="gameVersion"
								:color="currentGameVersion === gameVersion ? 'brand' : 'standard'"
							>
								<button
									v-tooltip="
										!possibleGameVersions.includes(gameVersion)
											? formatMessage(messages.gameVersionUnsupportedTooltip, {
													title: project.title,
													gameVersion: gameVersion,
													platform: currentPlatformText,
												})
											: null
									"
									:class="{
										'looks-disabled !text-brand-red':
											!possibleGameVersions.includes(gameVersion),
									}"
									@click="selectGameVersion(gameVersion)"
								>
									{{ gameVersion }}
									<CheckIcon v-if="userSelectedGameVersion === gameVersion" />
								</button>
							</ButtonStyled>
						</ScrollablePanel>
						<Checkbox
							v-if="showVersionsCheckbox"
							v-model="showAllVersions"
							class="mx-1"
							:label="formatMessage(messages.showAllVersions)"
							:disabled="!!versionFilter"
						/>
					</Accordion>
					<ButtonStyled v-if="project.loaders.length === 1 && project.project_type !== 'resourcepack'">
						<div class="disabled button-like">
							<WrenchIcon aria-hidden="true" />
							{{
								currentPlatform
									? formatMessage(messages.platformLabel, {
											platform: currentPlatformText,
										})
									: formatMessage(messages.platformError)
							}}
							<InfoIcon
								v-tooltip="
									formatMessage(messages.platformTooltip, {
										title: project.title,
										platform: currentPlatformText,
									})
								"
								class="ml-auto size-5"
							/>
						</div>
					</ButtonStyled>
					<Accordion
						v-else-if="project.project_type !== 'resourcepack'"
						ref="platformAccordion"
						class="accordion-with-bg"
						@on-open="
							() => {
								gameVersionAccordion?.close()
							}
						"
					>
						<template #title>
							<WrenchIcon aria-hidden="true" />
							{{
								currentPlatform
									? formatMessage(messages.platformLabel, {
											platform: currentPlatformText,
										})
									: formatMessage(messages.selectPlatform)
							}}
						</template>
						<ScrollablePanel :class="project.loaders.length > 4 ? 'h-[15rem]' : ''">
							<ButtonStyled
								v-for="platform in project.loaders.slice().reverse()"
								:key="platform"
								:color="currentPlatform === platform ? 'brand' : 'standard'"
							>
								<button
									v-tooltip="
										!possiblePlatforms.includes(platform)
											? formatMessage(messages.platformUnsupportedTooltip, {
													title: project.title,
													platform: currentPlatformText,
													gameVersion: currentGameVersion,
												})
											: null
									"
									:class="{
										'looks-disabled !text-brand-red': !possiblePlatforms.includes(platform),
									}"
									@click="selectPlatform(platform)"
								>
									{{ formatMessage(getTagMessage(platform, 'loader')) }}
									<CheckIcon v-if="userSelectedPlatform === platform" />
								</button>
							</ButtonStyled>
						</ScrollablePanel>
					</Accordion>
				</div>
				<AutomaticAccordion div class="flex flex-col gap-2">
					<VersionSummary
						v-if="filteredRelease"
						:version="filteredRelease"
						:decorate-download-url="decorateModalDownloadUrl"
						@on-download="onDownload"
						@on-navigate="onVersionNavigate"
					/>
					<VersionSummary
						v-if="filteredBeta"
						:version="filteredBeta"
						:decorate-download-url="decorateModalDownloadUrl"
						@on-download="onDownload"
						@on-navigate="onVersionNavigate"
					/>
					<VersionSummary
						v-if="filteredAlpha"
						:version="filteredAlpha"
						:decorate-download-url="decorateModalDownloadUrl"
						@on-download="onDownload"
						@on-navigate="onVersionNavigate"
					/>
					<p
						v-if="
							currentPlatform &&
							currentGameVersion &&
							!filteredRelease &&
							!filteredBeta &&
							!filteredAlpha &&
							!versionsLoading &&
							versions.length > 0
						"
					>
						{{
							formatMessage(messages.noVersionsAvailable, {
								gameVersion: currentGameVersion,
								platform: currentPlatformText,
							})
						}}
					</p>
				</AutomaticAccordion>
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
import {
	CheckIcon,
	ExternalIcon,
	GameIcon,
	InfoIcon,
	ModrinthIcon,
	SearchIcon,
	WrenchIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	Checkbox,
	commonMessages,
	defineMessages,
	getTagMessage,
	NewModal,
	ScrollablePanel,
	ServersPromo,
	StyledInput,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import VersionSummary from '@modrinth/ui/src/components/version/VersionSummary.vue'
import dayjs from 'dayjs'
import { computed, nextTick, ref, watch } from 'vue'

import { navigateTo } from '#app'
import Accordion from '~/components/ui/Accordion.vue'
import AutomaticAccordion from '~/components/ui/AutomaticAccordion.vue'
import { saveFeatureFlags } from '~/composables/featureFlags.ts'

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
const userSelectedGameVersion = ref(null)
const userSelectedPlatform = ref(null)
const showAllVersions = ref(false)
const versionFilter = ref('')
const gameVersionFilterInput = ref()
const gameVersionAccordion = ref()
const platformAccordion = ref()
const getModrinthAppAccordion = ref()

const currentGameVersion = computed(() => {
	return (
		userSelectedGameVersion.value ||
		(props.project.game_versions.length === 1 && props.project.game_versions[0])
	)
})

const possibleGameVersions = computed(() => {
	return props.versions
		.filter((x) => !currentPlatform.value || x.loaders.includes(currentPlatform.value))
		.flatMap((x) => x.game_versions)
})

const possiblePlatforms = computed(() => {
	return props.versions
		.filter((x) => !currentGameVersion.value || x.game_versions.includes(currentGameVersion.value))
		.flatMap((x) => x.loaders)
})

const currentPlatform = computed(() => {
	return (
		userSelectedPlatform.value ||
		(props.project.loaders.length === 1 && props.project.loaders[0])
	)
})

const currentPlatformText = computed(() => {
	if (!currentPlatform.value) return null
	return formatMessage(getTagMessage(currentPlatform.value, 'loader'))
})

const releaseVersions = computed(() => {
	const set = new Set()
	for (const gameVersion of props.tags.gameVersions || []) {
		if (gameVersion?.version && gameVersion.version_type === 'release') {
			set.add(gameVersion.version)
		}
	}
	return set
})

const nonReleaseVersions = computed(() => {
	const set = new Set()
	for (const gameVersion of props.tags.gameVersions || []) {
		if (gameVersion?.version && gameVersion.version_type !== 'release') {
			set.add(gameVersion.version)
		}
	}
	return set
})

const showVersionsCheckbox = computed(() => {
	let hasRelease = false
	let hasNonRelease = false

	for (const version of props.project.game_versions) {
		if (isReleaseGameVersion(version)) {
			hasRelease = true
		} else {
			hasNonRelease = true
		}

		if (hasRelease && hasNonRelease) return true
	}

	return false
})

const filteredGameVersions = computed(() => {
	return props.project.game_versions
		.filter(
			(x) =>
				(versionFilter.value && x.includes(versionFilter.value)) ||
				(!versionFilter.value && (showAllVersions.value || isReleaseGameVersion(x))),
		)
		.slice()
		.reverse()
})

const filteredVersions = computed(() => {
	const result = props.versions.filter(
		(x) =>
			x.game_versions?.includes(currentGameVersion.value) &&
			(x.loaders?.includes(currentPlatform.value) || props.project.project_type === 'resourcepack'),
	)
	debug('filteredVersions', {
		total: props.versions.length,
		filtered: result.length,
		currentGameVersion: currentGameVersion.value,
		currentPlatform: currentPlatform.value,
		sampleLoaders: props.versions.slice(0, 3).map((v) => v.loaders),
	})
	return result
})

const filteredRelease = computed(() => {
	return filteredVersions.value.find((x) => x.version_type === 'release')
})

const filteredBeta = computed(() => {
	return filteredVersions.value.find(
		(x) =>
			x.version_type === 'beta' &&
			(!filteredRelease.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredRelease.value.date_published))),
	)
})

const filteredAlpha = computed(() => {
	return filteredVersions.value.find(
		(x) =>
			x.version_type === 'alpha' &&
			(!filteredRelease.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredRelease.value.date_published))) &&
			(!filteredBeta.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredBeta.value.date_published))),
	)
})

const messages = defineMessages({
	dontHaveModrinthApp: {
		id: 'project.download.no-app',
		defaultMessage: "Don't have Modrinth App?",
	},
	downloadTitle: {
		id: 'project.download.title',
		defaultMessage: 'Download {title}',
	},
	gameVersionError: {
		id: 'project.download.game-version-error',
		defaultMessage: 'Error: no game versions found',
	},
	gameVersionLabel: {
		id: 'project.download.game-version',
		defaultMessage: 'Game version: {version}',
	},
	gameVersionTooltip: {
		id: 'project.download.game-version-tooltip',
		defaultMessage: '{title} is only available for {version}',
	},
	gameVersionUnsupportedTooltip: {
		id: 'project.download.game-version-unsupported-tooltip',
		defaultMessage: '{title} does not support {gameVersion} for {platform}',
	},
	installWithModrinthApp: {
		id: 'project.download.install-with-app',
		defaultMessage: 'Install with Modrinth App',
	},
	noVersionsAvailable: {
		id: 'project.download.no-versions-available',
		defaultMessage: 'No versions available for {gameVersion} and {platform}.',
	},
	platformError: {
		id: 'project.download.platform-error',
		defaultMessage: 'Error: no platforms found',
	},
	platformLabel: {
		id: 'project.download.platform',
		defaultMessage: 'Platform: {platform}',
	},
	platformTooltip: {
		id: 'project.download.platform-tooltip',
		defaultMessage: '{title} is only available for {platform}',
	},
	platformUnsupportedTooltip: {
		id: 'project.download.platform-unsupported-tooltip',
		defaultMessage: '{title} does not support {platform} for {gameVersion}',
	},
	searchGameVersions: {
		id: 'project.download.search-game-versions',
		defaultMessage: 'Search game versions...',
	},
	searchGameVersionsLabel: {
		id: 'project.download.search-game-versions-label',
		defaultMessage: 'Search game versions...',
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

function decorateModalDownloadUrl(url) {
	return createProjectDownloadUrl(url, {
		reason: props.downloadReason,
		gameVersion: currentGameVersion.value ?? undefined,
		loader: currentPlatform.value ?? undefined,
	})
}

function isReleaseGameVersion(version) {
	if (releaseVersions.value.has(version)) return true
	if (nonReleaseVersions.value.has(version)) return false
	return true
}

function updateDownloadQuery() {
	navigateTo(
		{
			query: {
				...route.query,
				...(userSelectedGameVersion.value && {
					version: userSelectedGameVersion.value,
				}),
				...(userSelectedPlatform.value && {
					loader: userSelectedPlatform.value,
				}),
			},
			hash: route.hash,
		},
		{ replace: true },
	)
}

function selectGameVersion(gameVersion) {
	userSelectedGameVersion.value = gameVersion
	gameVersionAccordion.value?.close()

	if (!currentPlatform.value) {
		platformAccordion.value?.open()
	}

	updateDownloadQuery()
}

function selectPlatform(platform) {
	userSelectedPlatform.value = platform
	platformAccordion.value?.close()

	if (!currentGameVersion.value) {
		gameVersionAccordion.value?.open()
	}

	updateDownloadQuery()
}

function installWithApp() {
	setTimeout(() => {
		getModrinthAppAccordion.value?.open()
	}, 1500)
}

function onShow() {
	debug('on-show fired')
	props.loadVersions()
	navigateTo({ query: route.query, hash: '#download' }, { replace: true })
}

function onHide() {
	navigateTo({ query: route.query, hash: '' }, { replace: true })
}

function show(event) {
	modal.value?.show(event)
}

function hide(event) {
	modal.value?.hide(event)
	userSelectedPlatform.value = null
	userSelectedGameVersion.value = null
	showAllVersions.value = false
}

function onDownload(event) {
	emit('download')
	setTimeout(() => {
		hide(event)
	}, 400)
}

function onVersionNavigate(url) {
	hide()
	nextTick(() => {
		navigateTo(url)
	})
}

function openFromHash() {
	if (!modal.value || route.hash !== '#download') return

	debug('hash #download watch fired, opening modal')
	props.loadVersions()
	modal.value.show()
}

const { version, loader } = route.query

if (
	props.project.game_versions.length > 0 &&
	props.project.game_versions.every((projectVersion) => !isReleaseGameVersion(projectVersion))
) {
	showAllVersions.value = true
}

if (version !== undefined && props.project.game_versions.includes(version)) {
	userSelectedGameVersion.value = version
}

if (loader !== undefined && props.project.loaders.includes(loader)) {
	userSelectedPlatform.value = loader
}

if (route.hash === '#download' || version !== undefined || loader !== undefined) {
	debug('eager loadVersions from setup', { hash: route.hash, version, loader })
	props.loadVersions()
}

watch(modal, openFromHash)
watch(() => route.hash, openFromHash)

defineExpose({ show, hide })
</script>

<style lang="scss" scoped>
:deep(.accordion-with-bg) {
	@apply rounded-2xl bg-bg p-2;
	--scrollable-pane-bg: var(--color-bg);
}

@media (hover: none) and (max-width: 767px) {
	.modrinth-app-section {
		display: none;
	}
}
</style>
