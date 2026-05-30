<template>
	<NewModal
		ref="incompatibleModal"
		no-padding
		fade="warning"
		max-width="600px"
		width="600px"
		:on-hide="handleHide"
		:header="formatMessage(messages.header)"
	>
		<div class="flex h-[min(680px,calc(95vh-8rem))] min-h-0 flex-col overflow-hidden">
			<div class="flex flex-col gap-4 p-6 pb-4">
				<Admonition
					type="warning"
					:body="
						formatMessage(messages.conflictSummary, {
							instance: formatInstanceCompatibilityLabel(),
						})
					"
				/>

				<StyledInput
					v-model="search"
					:icon="SearchIcon"
					type="search"
					clearable
					:placeholder="formatMessage(messages.searchPlaceholder)"
					wrapper-class="w-full"
				/>
			</div>

			<div class="relative min-h-0 flex-1">
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-4"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-4"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showVersionTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-4 bg-gradient-to-b from-bg-raised to-transparent"
					/>
				</Transition>

				<div
					ref="versionListRef"
					class="h-full min-h-0 overflow-y-auto px-6 pb-3"
					@scroll="checkVersionScrollState"
				>
					<div v-if="filteredVersions.length > 0" class="flex flex-col gap-2" role="listbox">
						<div
							v-for="version in filteredVersions"
							:key="version.id"
							role="option"
							tabindex="0"
							:aria-selected="selectedVersion?.id === version.id"
							class="w-full cursor-pointer rounded-2xl border border-solid p-4 text-left active:scale-[0.99] focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
							:class="
								selectedVersion?.id === version.id
									? 'border-brand bg-brand-highlight'
									: 'border-surface-5 bg-surface-3 hover:brightness-[1.08] focus-visible:brightness-[1.08]'
							"
							@click="selectedVersion = version"
							@keydown.enter.prevent="selectedVersion = version"
							@keydown.space.prevent="selectedVersion = version"
						>
							<div class="flex items-start gap-3">
								<VersionChannelIndicator
									v-if="version.version_type"
									:channel="version.version_type"
									size="sm"
									class="mt-0.5 shrink-0"
								/>
								<div class="min-w-0 flex-1">
									<div class="flex items-center gap-2">
										<span class="truncate font-semibold text-contrast">
											{{ version.name || version.version_number }}
										</span>
									</div>

									<div class="mt-2 flex flex-wrap items-center gap-2">
										<span v-if="formatVersionDate(version)" class="font-medium">
											{{ formatVersionDate(version) }}
										</span>
										<BulletDivider
											v-if="formatVersionDate(version)"
											:class="isSelectedVersion(version) ? '!bg-brand-highlight' : ''"
										/>
										<span class="font-medium">
											{{
												formatMessage(messages.downloadsLabel, {
													downloads: formatCompactNumber(version.downloads ?? 0),
												})
											}}
										</span>
									</div>

									<div class="mt-3 flex flex-wrap gap-1.5">
										<span
											v-for="gameVersion in getVisibleGameVersions(version)"
											:key="`${version.id}-${gameVersion}`"
											class="inline-flex items-center rounded-full border border-solid px-2 py-1 leading-none"
											:class="
												isSelectedVersion(version) || isMatchingGameVersion(version, gameVersion)
													? 'border-brand bg-brand-highlight text-brand'
													: 'border-surface-5 bg-surface-4'
											"
										>
											{{ gameVersion }}
										</span>
										<Tooltip
											v-if="getHiddenGameVersions(version).length > 0"
											theme="dismissable-prompt"
											class="inline-flex shrink-0 items-center"
											:triggers="['hover', 'focus']"
											:popper-triggers="['hover', 'focus']"
											popper-class="v-popper--interactive game-version-list-popper"
											placement="top"
											:delay="{ show: 200, hide: 100 }"
											no-auto-focus
										>
											<button
												type="button"
												class="inline-flex cursor-help items-center rounded-full border border-solid border-surface-5 bg-surface-4 px-2 py-1 leading-none"
												:aria-label="formatHiddenGameVersionsLabel(version)"
												@click.stop
												@keydown.stop
											>
												{{
													formatMessage(messages.moreGameVersions, {
														count: getHiddenGameVersions(version).length,
													})
												}}
											</button>
											<template #popper>
												<div class="flex max-w-[22rem] flex-wrap gap-1.5">
													<span
														v-for="gameVersion in getHiddenGameVersions(version)"
														:key="`${version.id}-hidden-${gameVersion}`"
														class="inline-flex items-center rounded-full border border-solid px-2 py-1 leading-none"
														:class="
															isSelectedVersion(version) ||
															isMatchingGameVersion(version, gameVersion)
																? 'border-brand bg-brand-highlight text-brand'
																: 'border-surface-5 bg-surface-4'
														"
													>
														{{ gameVersion }}
													</span>
												</div>
											</template>
										</Tooltip>
									</div>
								</div>
							</div>
						</div>
					</div>
					<div v-else class="py-8 text-center text-sm text-secondary">
						{{ formatMessage(messages.noVersions) }}
					</div>
				</div>

				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-4"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-4"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showVersionBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-4 bg-gradient-to-t from-bg-raised to-transparent"
					/>
				</Transition>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="hideModal">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button :disabled="installing || !selectedVersion" @click="install()">
						<DownloadIcon />
						{{
							installing
								? formatMessage(commonMessages.installingLabel)
								: formatMessage(messages.installAnywayButton)
						}}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup>
import { DownloadIcon, SearchIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	BulletDivider,
	ButtonStyled,
	NewModal,
	StyledInput,
	VersionChannelIndicator,
	commonMessages,
	defineMessages,
	formatLoader,
	injectNotificationManager,
	useCompactNumber,
	useRelativeTime,
	useScrollIndicator,
	useVIntl,
} from '@modrinth/ui'
import { formatVersionsForDisplay } from '@modrinth/utils'
import { Tooltip } from 'floating-vue'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import { add_project_from_version as installMod } from '@/helpers/profile'
import { get_game_versions } from '@/helpers/tags'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const { formatCompactNumber } = useCompactNumber()

const instance = ref(null)
const project = ref(null)
const versions = ref(null)
const selectedVersion = ref(null)
const incompatibleModal = useTemplateRef('incompatibleModal')
const versionListRef = ref(null)
const installing = ref(false)
const installed = ref(false)
const search = ref('')
const gameVersionTags = ref([])
let gameVersionTagsPromise = null

const onInstall = ref(() => {})
const {
	showTopFade: showVersionTopFade,
	showBottomFade: showVersionBottomFade,
	checkScrollState: checkVersionScrollState,
	forceCheck: forceVersionScrollCheck,
} = useScrollIndicator(versionListRef)

const filteredVersions = computed(() => {
	const query = search.value.trim().toLowerCase()
	if (!query) return versions.value ?? []

	return (versions.value ?? []).filter((version) => {
		const searchable = [
			version.name,
			version.version_number,
			...version.loaders.map((loader) => formatLoader(formatMessage, loader)),
			...version.game_versions,
		]
			.filter(Boolean)
			.join(' ')
			.toLowerCase()

		return searchable.includes(query)
	})
})

function formatVersionDate(version) {
	if (!version.date_published) return ''
	return formatRelativeTime(new Date(version.date_published))
}

function formatInstanceCompatibilityLabel() {
	if (
		project.value?.project_type === 'resourcepack' ||
		project.value?.project_type === 'datapack'
	) {
		return instance.value?.game_version ?? ''
	}

	return `${formatLoader(formatMessage, instance.value?.loader ?? '')} ${
		instance.value?.game_version ?? ''
	}`.trim()
}

function formatHiddenGameVersionsLabel(version) {
	return formatMessage(messages.moreGameVersionsTooltip, {
		versions: getHiddenGameVersions(version).join(', '),
	})
}

function getDisplayGameVersions(version) {
	if (!gameVersionTags.value.length) return version.game_versions
	return formatVersionsForDisplay(version.game_versions, gameVersionTags.value).map((gameVersion) =>
		normalizeCompletePatchRange(gameVersion, version.game_versions),
	)
}

function getVisibleGameVersions(version) {
	return getDisplayGameVersions(version).slice(0, 5)
}

function getHiddenGameVersions(version) {
	return getDisplayGameVersions(version).slice(5)
}

function isSelectedVersion(version) {
	return selectedVersion.value?.id === version.id
}

function isMatchingGameVersion(version, gameVersion) {
	if (gameVersion === instance.value?.game_version) return true
	if (!gameVersion.endsWith('.x')) return false

	const prefix = gameVersion.slice(0, -2)
	const instanceGameVersion = instance.value?.game_version
	return (
		version.game_versions.includes(instanceGameVersion) &&
		(instanceGameVersion === prefix || instanceGameVersion?.startsWith(`${prefix}.`))
	)
}

const minecraftPatchRangeRegex = /^([0-9]+\.[0-9]+)\.([0-9]+)–\1\.([0-9]+)$/
const minecraftVersionRegex = /^([0-9]+\.[0-9]+)(?:\.([0-9]+))?$/

function normalizeCompletePatchRange(gameVersion, rawGameVersions) {
	const rangeMatch = gameVersion.match(minecraftPatchRangeRegex)
	if (!rangeMatch) return gameVersion

	const majorVersion = rangeMatch[1]
	const rawGameVersionSet = new Set(rawGameVersions)
	const patchVersionsForMajor = gameVersionTags.value
		.filter((tag) => tag.version_type === 'release')
		.map((tag) => {
			const versionMatch = tag.version.match(minecraftVersionRegex)
			if (!versionMatch || versionMatch[1] !== majorVersion) return null
			const patchVersion = Number(versionMatch[2] ?? 0)
			return patchVersion > 0 ? tag.version : null
		})
		.filter(Boolean)

	if (
		patchVersionsForMajor.length > 0 &&
		patchVersionsForMajor.every((version) => rawGameVersionSet.has(version))
	) {
		return `${majorVersion}.x`
	}

	return gameVersion
}

async function loadGameVersionTags() {
	if (gameVersionTags.value.length) return
	gameVersionTagsPromise ??= get_game_versions()
		.then((tags) => {
			gameVersionTags.value = tags ?? []
		})
		.catch(() => {
			gameVersionTags.value = []
		})
		.finally(() => {
			gameVersionTagsPromise = null
		})

	await gameVersionTagsPromise
}

function hideModal() {
	incompatibleModal.value?.hide()
}

async function showModal() {
	await nextTick()

	if (!incompatibleModal.value) {
		await new Promise((resolve) => requestAnimationFrame(resolve))
	}

	incompatibleModal.value?.show()
	await nextTick()
	forceVersionScrollCheck()
}

watch(filteredVersions, () => {
	nextTick(() => forceVersionScrollCheck())
})

defineExpose({
	show: async (instanceVal, projectVal, projectVersions, selected, callback) => {
		instance.value = instanceVal
		versions.value = projectVersions ?? []
		selectedVersion.value = selected ?? projectVersions?.[0] ?? null
		search.value = ''

		project.value = projectVal

		onInstall.value = callback
		installing.value = false
		installed.value = false

		await loadGameVersionTags()
		await showModal()

		trackEvent('ProjectInstallStart', { source: 'ProjectIncompatibilityWarningModal' })
	},
})

const handleHide = () => {
	if (!installed.value) {
		onInstall.value()
	}
	installed.value = false
}

const install = async () => {
	if (!selectedVersion.value) return
	installing.value = true
	try {
		await installMod(instance.value.path, selectedVersion.value.id, 'standalone')
	} catch (err) {
		handleError(err)
		installing.value = false
		return
	}
	installing.value = false
	installed.value = true
	onInstall.value(selectedVersion.value.id)
	hideModal()

	trackEvent('ProjectInstall', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
		id: project.value.id,
		version_id: selectedVersion.value.id,
		project_type: project.value.project_type,
		title: project.value.title,
		source: 'ProjectIncompatibilityWarningModal',
	})
}

const messages = defineMessages({
	header: {
		id: 'app.install.incompatibility-warning.header',
		defaultMessage: 'Incompatibility warning',
	},
	conflictSummary: {
		id: 'app.install.incompatibility-warning.conflict-summary',
		defaultMessage:
			'No available versions match {instance}. Select a version to install anyway. Dependencies will not be installed automatically.',
	},
	searchPlaceholder: {
		id: 'app.install.incompatibility-warning.search-placeholder',
		defaultMessage: 'Search by version or game version...',
	},
	downloadsLabel: {
		id: 'app.install.incompatibility-warning.downloads',
		defaultMessage: '{downloads} downloads',
	},
	moreGameVersions: {
		id: 'app.install.incompatibility-warning.more-game-versions',
		defaultMessage: '+{count} more',
	},
	moreGameVersionsTooltip: {
		id: 'app.install.incompatibility-warning.more-game-versions-tooltip',
		defaultMessage: 'More game versions: {versions}',
	},
	noVersions: {
		id: 'app.install.incompatibility-warning.no-versions',
		defaultMessage: 'No versions found',
	},
	installAnywayButton: {
		id: 'app.install.incompatibility-warning.install-anyway',
		defaultMessage: 'Install anyway',
	},
})
</script>
