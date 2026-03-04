<template>
	<div class="flex flex-col gap-6 rounded-2xl bg-surface-3 p-6">
		<div v-if="!server" class="flex items-center justify-center py-12">
			<SpinnerIcon class="size-8 animate-spin text-secondary" />
		</div>
		<template v-else>
			<div class="flex flex-col gap-2.5">
				<span class="text-lg font-semibold text-contrast">{{
					formatMessage(commonMessages.installationInfoTitle)
				}}</span>
				<div class="flex flex-col gap-2.5 rounded-[20px] bg-surface-2 p-4">
					<div
						v-for="row in installationInfo"
						:key="row.label"
						class="flex items-center justify-between"
					>
						<span class="text-primary">{{ row.label }}</span>
						<span class="font-semibold text-contrast">{{ row.value }}</span>
					</div>
				</div>
			</div>

			<div v-if="addonsQuery.isLoading.value" class="flex items-center justify-center py-6">
				<SpinnerIcon class="size-6 animate-spin text-secondary" />
			</div>

			<template v-else-if="isLinked">
				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(commonMessages.installedModpackTitle)
					}}</span>
					<div v-if="modpack" class="flex items-center gap-2.5 rounded-[20px] bg-surface-2 p-3">
						<AutoLink :to="`/project/${modpack.spec.project_id}`" class="shrink-0">
							<div
								class="size-14 shrink-0 overflow-hidden rounded-2xl border border-solid border-surface-5"
							>
								<Avatar
									v-if="modpack.icon_url"
									:src="modpack.icon_url"
									:alt="modpack.title ?? formatMessage(commonMessages.modpackLabel)"
									size="100%"
									no-shadow
								/>
							</div>
						</AutoLink>
						<div class="flex flex-col gap-1">
							<AutoLink
								:to="`/project/${modpack.spec.project_id}`"
								class="font-semibold text-contrast hover:underline"
							>
								{{ modpack.title ?? modpack.spec.project_id }}
							</AutoLink>
							<div class="flex items-center gap-2 text-sm text-secondary">
								<AutoLink
									v-if="modpack.owner"
									:to="
										modpack.owner.type === 'organization'
											? `/organization/${modpack.owner.id}`
											: `/user/${modpack.owner.id}`
									"
									class="flex items-center gap-1.5 hover:underline"
								>
									<Avatar
										:src="modpack.owner.icon_url"
										:alt="modpack.owner.name"
										size="1.25rem"
										:circle="modpack.owner.type === 'user'"
										no-shadow
									/>
									<span class="font-medium">{{ modpack.owner.name }}</span>
								</AutoLink>
								<template v-if="modpack.owner && modpack.version_number"> &middot; </template>
								<span v-if="modpack.version_number" class="font-medium">{{
									modpack.version_number
								}}</span>
							</div>
						</div>
					</div>
					<div class="flex flex-wrap gap-2">
						<ButtonStyled>
							<button
								class="!shadow-none"
								:disabled="isInstalling"
								@click="handleChangeModpackVersion"
							>
								<ArrowLeftRightIcon class="size-5" />
								{{ formatMessage(commonMessages.changeVersionButton) }}
							</button>
						</ButtonStyled>
						<ButtonStyled color="orange">
							<button class="!shadow-none" :disabled="isInstalling" @click="repairModal?.show()">
								<HammerIcon class="size-5" />
								{{ formatMessage(commonMessages.repairButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>

				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(messages.linkedInstanceTitle)
					}}</span>
					<span class="text-primary">
						{{ formatMessage(messages.unlinkDescription) }}
					</span>
					<div>
						<ButtonStyled color="orange">
							<button class="!shadow-none" :disabled="isInstalling" @click="unlinkModal?.show()">
								<UnlinkIcon class="size-5" />
								{{ formatMessage(commonMessages.unlinkModpackButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>

				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(messages.reinstallModpackTitle)
					}}</span>
					<span class="text-primary">
						{{ formatMessage(messages.reinstallModpackDescription) }}
					</span>
					<div>
						<ButtonStyled color="red">
							<button class="!shadow-none" :disabled="isInstalling" @click="reinstallModal?.show()">
								<DownloadIcon class="size-5" />
								{{ formatMessage(commonMessages.reinstallModpackButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>

				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(messages.resetServerTitle)
					}}</span>
					<span class="text-primary">
						{{ formatMessage(messages.resetServerDescription) }}
					</span>
					<div>
						<ButtonStyled color="red">
							<button class="!shadow-none" :disabled="isInstalling" @click="setupModal?.show()">
								<RotateCounterClockwiseIcon class="size-5" />
								{{ formatMessage(commonMessages.resetServerButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</template>

			<template v-else>
				<div v-if="isEditing" class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(messages.editInstallationTitle)
					}}</span>
					<div class="flex flex-col gap-3 rounded-[20px] border border-solid border-surface-5 p-4">
						<div class="flex flex-col gap-2.5">
							<span class="font-semibold text-contrast">{{
								formatMessage(commonMessages.platformLabel)
							}}</span>
							<Chips
								v-model="selectedPlatform"
								:items="availablePlatforms"
								:aria-label="formatMessage(messages.selectPlatformAriaLabel)"
							/>
						</div>

						<div class="flex flex-col gap-2.5">
							<span class="font-semibold text-contrast">{{
								formatMessage(commonMessages.gameVersionLabel)
							}}</span>
							<Combobox
								v-model="selectedGameVersion"
								:options="gameVersionOptions"
								searchable
								sync-with-selection
								:placeholder="formatMessage(commonMessages.selectVersionPlaceholder)"
								:search-placeholder="formatMessage(messages.searchGameVersionPlaceholder)"
								:display-value="
									selectedGameVersion || formatMessage(commonMessages.selectVersionPlaceholder)
								"
								:aria-label="formatMessage(messages.selectGameVersionAriaLabel)"
							>
								<template v-if="hasSnapshots" #dropdown-footer>
									<button
										class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
										@mousedown.prevent
										@click="showSnapshots = !showSnapshots"
									>
										<EyeOffIcon v-if="showSnapshots" class="size-4" />
										<EyeIcon v-else class="size-4" />
										{{
											showSnapshots
												? formatMessage(commonMessages.hideSnapshotsButton)
												: formatMessage(commonMessages.showAllVersionsButton)
										}}
									</button>
								</template>
							</Combobox>
						</div>

						<div v-if="selectedPlatform !== 'vanilla'" class="flex flex-col gap-2.5">
							<span class="font-semibold text-contrast">{{
								formatMessage(messages.loaderVersionLabel, { loader: formattedLoaderName })
							}}</span>
							<Combobox
								v-model="selectedLoaderVersion"
								searchable
								sync-with-selection
								:placeholder="loaderVersionDisplayValue"
								:search-placeholder="formatMessage(commonMessages.searchVersionPlaceholder)"
								:options="loaderVersionOptions"
								:display-value="loaderVersionDisplayValue"
								:aria-label="
									formatMessage(messages.selectLoaderVersionAriaLabel, {
										loader: formattedLoaderName,
									})
								"
							/>
						</div>

						<div class="flex flex-wrap gap-2">
							<ButtonStyled color="brand">
								<button
									class="!shadow-none"
									:disabled="!isValid || !hasChanges || isSaving"
									@click="handleSave"
								>
									<SpinnerIcon v-if="isSaving" class="animate-spin" />
									<SaveIcon v-else />
									{{
										isSaving
											? formatMessage(messages.savingLabel)
											: formatMessage(commonMessages.saveButton)
									}}
								</button>
							</ButtonStyled>
							<ButtonStyled type="outlined">
								<button class="!border !border-surface-5 !shadow-none" @click="cancelEditing">
									<XIcon />
									{{ formatMessage(commonMessages.cancelButton) }}
								</button>
							</ButtonStyled>
						</div>
					</div>
				</div>

				<template v-if="!isEditing">
					<div class="flex items-start gap-2">
						<CircleAlertIcon class="mt-0.5 size-5 shrink-0 text-orange" />
						<span class="text-primary">
							{{ formatMessage(messages.editWarning) }}
						</span>
					</div>

					<div class="flex flex-wrap gap-2">
						<ButtonStyled color="orange">
							<button class="!shadow-none" :disabled="isInstalling" @click="isEditing = true">
								<PencilIcon class="size-5" />
								{{ formatMessage(commonMessages.editButton) }}
							</button>
						</ButtonStyled>
						<ButtonStyled>
							<button class="!shadow-none" :disabled="isInstalling" @click="setupModal?.show()">
								{{ formatMessage(commonMessages.resetServerButton) }}
								<ChevronRightIcon class="size-5" />
							</button>
						</ButtonStyled>
					</div>
				</template>

				<div class="flex flex-col gap-2.5">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(messages.repairServerTitle)
					}}</span>
					<span class="text-primary">
						{{ formatMessage(messages.repairServerDescription) }}
					</span>
					<div>
						<ButtonStyled color="orange">
							<button class="!shadow-none" :disabled="isInstalling" @click="repairModal?.show()">
								<HammerIcon class="size-5" />
								{{ formatMessage(commonMessages.repairButton) }}
							</button>
						</ButtonStyled>
					</div>
				</div>
			</template>
		</template>
	</div>
	<ConfirmUnlinkModal ref="unlinkModal" server @unlink="handleUnlinkConfirm" />
	<ServerSetupModal
		ref="setupModal"
		@reinstall="emit('reinstall', $event)"
		@browse-modpacks="onBrowseModpacks"
	/>
	<ContentUpdaterModal
		v-if="updatingModpack"
		ref="contentUpdaterModal"
		:versions="updatingProjectVersions"
		:current-game-version="currentGameVersion"
		:current-loader="currentLoader"
		:current-version-id="modpack?.spec.version_id ?? ''"
		:is-app="false"
		:is-modpack="true"
		:project-icon-url="modpack?.icon_url ?? undefined"
		:project-name="
			modpack?.title ?? modpack?.spec.project_id ?? formatMessage(commonMessages.modpackLabel)
		"
		:loading="loadingVersions"
		:loading-changelog="loadingChangelog"
		@update="handleUpdaterConfirm"
		@cancel="resetUpdateState"
		@version-select="handleUpdaterVersionSelect"
		@version-hover="handleUpdaterVersionHover"
	/>
	<ConfirmRepairModal ref="repairModal" server @repair="handleRepair" />
	<ConfirmReinstallModal ref="reinstallModal" @reinstall="handleReinstallConfirm" />
</template>

<script setup lang="ts">
import type { Archon, Labrinth, LauncherMeta } from '@modrinth/api-client'
import {
	ArrowLeftRightIcon,
	ChevronRightIcon,
	CircleAlertIcon,
	DownloadIcon,
	EyeIcon,
	EyeOffIcon,
	HammerIcon,
	PencilIcon,
	RotateCounterClockwiseIcon,
	SaveIcon,
	SpinnerIcon,
	UnlinkIcon,
	XIcon,
} from '@modrinth/assets'
import {
	AutoLink,
	Avatar,
	ButtonStyled,
	Chips,
	Combobox,
	commonMessages,
	ConfirmReinstallModal,
	ConfirmRepairModal,
	ConfirmUnlinkModal,
	ContentUpdaterModal,
	defineMessages,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	injectTags,
	ServerSetupModal,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref, watch } from 'vue'

const client = injectModrinthClient()
const { server, serverId, worldId } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const tags = injectTags()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	editInstallationTitle: {
		id: 'hosting.loader.edit-installation',
		defaultMessage: 'Edit installation',
	},
	linkedInstanceTitle: {
		id: 'hosting.loader.linked-instance',
		defaultMessage: 'Linked instance',
	},
	unlinkDescription: {
		id: 'hosting.loader.unlink-description',
		defaultMessage:
			"Unlinking permanently disconnects this instance from the modpack project, allowing you to change the loader and Minecraft version, but you won't receive future updates.",
	},
	reinstallModpackTitle: {
		id: 'hosting.loader.reinstall-modpack',
		defaultMessage: 'Re-install modpack',
	},
	reinstallModpackDescription: {
		id: 'hosting.loader.reinstall-modpack-description',
		defaultMessage:
			"Re-installing the modpack resets the instance's content to its original state, removing any mods or content you have added.",
	},
	resetServerTitle: {
		id: 'hosting.loader.reset-server',
		defaultMessage: 'Reset server',
	},
	resetServerDescription: {
		id: 'hosting.loader.reset-server-description',
		defaultMessage:
			'Removes all data on your server, including your worlds, mods, and configuration files. Backups will remain and can be restored.',
	},
	repairServerTitle: {
		id: 'hosting.loader.repair-server',
		defaultMessage: 'Repair server',
	},
	repairServerDescription: {
		id: 'hosting.loader.repair-server-description',
		defaultMessage:
			'Reinstalls the loader and Minecraft dependencies without deleting your content. This may resolve issues if your server is not starting correctly.',
	},
	editWarning: {
		id: 'hosting.loader.edit-warning',
		defaultMessage:
			"We don't recommend editing your installation settings after installing content. If you want to edit them reset your server.",
	},
	loaderVersionLabel: {
		id: 'hosting.loader.loader-version',
		defaultMessage: '{loader} version',
	},
	searchGameVersionPlaceholder: {
		id: 'hosting.loader.search-game-version',
		defaultMessage: 'Search game version...',
	},
	savingLabel: {
		id: 'hosting.loader.saving',
		defaultMessage: 'Saving...',
	},
	failedToLoadVersions: {
		id: 'hosting.loader.failed-to-load-versions',
		defaultMessage: 'Failed to load versions',
	},
	failedToChangeVersion: {
		id: 'hosting.loader.failed-to-change-version',
		defaultMessage: 'Failed to change modpack version',
	},
	failedToSaveSettings: {
		id: 'hosting.loader.failed-to-save-settings',
		defaultMessage: 'Failed to save installation settings',
	},
	failedToRepair: {
		id: 'hosting.loader.failed-to-repair',
		defaultMessage: 'Failed to repair server',
	},
	failedToReinstall: {
		id: 'hosting.loader.failed-to-reinstall',
		defaultMessage: 'Failed to reinstall modpack',
	},
	failedToUnlink: {
		id: 'hosting.loader.failed-to-unlink',
		defaultMessage: 'Failed to unlink modpack',
	},
	selectPlatformAriaLabel: {
		id: 'hosting.loader.aria.select-platform',
		defaultMessage: 'Select platform',
	},
	selectGameVersionAriaLabel: {
		id: 'hosting.loader.aria.select-game-version',
		defaultMessage: 'Select game version',
	},
	selectLoaderVersionAriaLabel: {
		id: 'hosting.loader.aria.select-loader-version',
		defaultMessage: 'Select {loader} version',
	},
})

const emit = defineEmits<{
	reinstall: [any?]
}>()

const isInstalling = computed(() => server.value?.status === 'installing')
const isEditing = ref(false)

const addonsQuery = useQuery({
	queryKey: computed(() => ['content', 'list', 'v1', serverId]),
	queryFn: () =>
		client.archon.content_v1.getAddons(serverId, worldId.value!, { from_modpack: false }),
	enabled: computed(() => worldId.value !== null),
})

const modpack = computed(() => addonsQuery.data.value?.modpack ?? null)
const isLinked = computed(() => !!modpack.value)

const modpackVersionsQuery = useQuery({
	queryKey: computed(() => ['labrinth', 'versions', 'v2', modpack.value?.spec.project_id]),
	queryFn: () =>
		client.labrinth.versions_v2.getProjectVersions(modpack.value!.spec.project_id, {
			include_changelog: false,
		}),
	enabled: computed(() => !!modpack.value?.spec.project_id),
})

function capitalize(str: string): string {
	return str.charAt(0).toUpperCase() + str.slice(1)
}

const installationInfo = computed(() => {
	const addons = addonsQuery.data.value
	const unknownStr = formatMessage(commonMessages.unknownLabel)
	const rawLoader = addons?.modloader ?? server.value?.loader ?? unknownStr
	const loader = capitalize(rawLoader)
	const gameVersion = addons?.game_version ?? server.value?.mc_version ?? unknownStr
	const loaderVersion = addons?.modloader_version ?? server.value?.loader_version ?? unknownStr

	const rows = [
		{ label: formatMessage(commonMessages.platformLabel), value: loader },
		{ label: formatMessage(commonMessages.gameVersionLabel), value: gameVersion },
	]
	if (loader && loader !== 'Vanilla') {
		rows.push({
			label: formatMessage(messages.loaderVersionLabel, { loader }),
			value: loaderVersion,
		})
	}
	return rows
})

async function handleChangeModpackVersion() {
	if (!modpack.value?.spec.project_id) return

	updatingModpack.value = true
	loadingChangelog.value = false

	const cached = modpackVersionsQuery.data.value
	if (cached) {
		const sorted = [...cached].sort(
			(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
		)
		updatingProjectVersions.value = sorted
		loadingVersions.value = false
	} else {
		updatingProjectVersions.value = []
		loadingVersions.value = true
	}

	await nextTick()

	contentUpdaterModal.value?.show(modpack.value.spec.version_id ?? undefined)

	if (!cached) {
		try {
			const fetchedVersions = await client.labrinth.versions_v2.getProjectVersions(
				modpack.value.spec.project_id,
				{ include_changelog: false },
			)
			fetchedVersions.sort(
				(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
			)
			updatingProjectVersions.value = fetchedVersions
		} catch (err) {
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : formatMessage(messages.failedToLoadVersions),
			})
		} finally {
			loadingVersions.value = false
		}
	}
}

async function handleUpdaterVersionSelect(version: Labrinth.Versions.v2.Version) {
	if (version.changelog !== undefined) return
	loadingChangelog.value = true
	try {
		const fullVersion = await client.labrinth.versions_v2.getVersion(version.id)
		const index = updatingProjectVersions.value.findIndex((v) => v.id === version.id)
		if (index !== -1) {
			const newVersions = [...updatingProjectVersions.value]
			newVersions[index] = fullVersion
			updatingProjectVersions.value = newVersions
		}
	} catch {
		// Silently fail on changelog fetch
	} finally {
		loadingChangelog.value = false
	}
}

async function handleUpdaterVersionHover(version: Labrinth.Versions.v2.Version) {
	if (version.changelog !== undefined) return
	try {
		const fullVersion = await client.labrinth.versions_v2.getVersion(version.id)
		const index = updatingProjectVersions.value.findIndex((v) => v.id === version.id)
		if (index !== -1) {
			const newVersions = [...updatingProjectVersions.value]
			newVersions[index] = fullVersion
			updatingProjectVersions.value = newVersions
		}
	} catch {
		// Silently fail on hover prefetch
	}
}

function resetUpdateState() {
	updatingModpack.value = false
	updatingProjectVersions.value = []
	loadingVersions.value = false
	loadingChangelog.value = false
}

async function handleUpdaterConfirm(selectedVersion: Labrinth.Versions.v2.Version) {
	if (!modpack.value) return
	try {
		await client.archon.content_v1.installContent(serverId, worldId.value!, {
			content_variant: 'modpack',
			spec: {
				platform: 'modrinth',
				project_id: modpack.value.spec.project_id,
				version_id: selectedVersion.id,
			},
			soft_override: true,
		})
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		emit('reinstall')
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : formatMessage(messages.failedToChangeVersion),
		})
	} finally {
		resetUpdateState()
	}
}

function onBrowseModpacks() {
	navigateTo({
		path: '/discover/modpacks',
		query: { sid: serverId, from: 'reset-server', wid: worldId.value },
	})
}

const unlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
const setupModal = ref<InstanceType<typeof ServerSetupModal>>()
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal>>()
const reinstallModal = ref<InstanceType<typeof ConfirmReinstallModal>>()
const repairModal = ref<InstanceType<typeof ConfirmRepairModal>>()

const updatingModpack = ref(false)
const updatingProjectVersions = ref<Labrinth.Versions.v2.Version[]>([])
const loadingVersions = ref(false)
const loadingChangelog = ref(false)

const currentGameVersion = computed(
	() => addonsQuery.data.value?.game_version ?? server.value?.mc_version ?? '',
)
const currentLoader = computed(
	() => addonsQuery.data.value?.modloader ?? server.value?.loader ?? '',
)

const availablePlatforms = ['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']

const selectedPlatform = ref(server.value?.loader?.toLowerCase() ?? 'vanilla')
const selectedGameVersion = ref(server.value?.mc_version ?? '')
const selectedLoaderVersion = ref<number>(0)
const showSnapshots = ref(false)
const isSaving = ref(false)

type LoaderVersionEntry = LauncherMeta.Manifest.v0.LoaderVersion

const modLoaders = ['fabric', 'forge', 'quilt', 'neoforge']

function toApiLoaderName(loader: string): string {
	return loader === 'neoforge' ? 'neo' : loader
}

const apiLoaderName = computed(() =>
	modLoaders.includes(selectedPlatform.value) ? toApiLoaderName(selectedPlatform.value) : null,
)

const manifestQuery = useQuery({
	queryKey: computed(() => ['loader-manifest', apiLoaderName.value] as const),
	queryFn: () => client.launchermeta.manifest_v0.getManifest(apiLoaderName.value!),
	enabled: computed(() => !!apiLoaderName.value),
	staleTime: 5 * 60 * 1000,
})

const paperBuildsQuery = useQuery({
	queryKey: computed(() => ['paper-builds', selectedGameVersion.value] as const),
	queryFn: () => client.paper.versions_v3.getBuilds(selectedGameVersion.value),
	enabled: computed(() => selectedPlatform.value === 'paper' && !!selectedGameVersion.value),
	staleTime: 5 * 60 * 1000,
})

const purpurBuildsQuery = useQuery({
	queryKey: computed(() => ['purpur-builds', selectedGameVersion.value] as const),
	queryFn: () => client.purpur.versions_v2.getBuilds(selectedGameVersion.value),
	enabled: computed(() => selectedPlatform.value === 'purpur' && !!selectedGameVersion.value),
	staleTime: 5 * 60 * 1000,
})

function getLoaderVersionsForGameVersion(
	loader: string,
	gameVersion: string,
): LoaderVersionEntry[] {
	if (loader === 'paper') {
		return (paperBuildsQuery.data.value?.builds ?? [])
			.toSorted((a, b) => b - a)
			.map((b) => ({ id: String(b), stable: true }))
	}
	if (loader === 'purpur') {
		return (purpurBuildsQuery.data.value?.builds.all ?? [])
			.toSorted((a, b) => parseInt(b) - parseInt(a))
			.map((b) => ({ id: b, stable: true }))
	}

	const manifest = manifestQuery.data.value?.gameVersions
	if (!manifest) return []

	const placeholder = manifest.find((x) => x.id === '${modrinth.gameVersion}')
	if (placeholder) return placeholder.loaders

	const entry = manifest.find((x) => x.id === gameVersion)
	return entry?.loaders ?? []
}

const gameVersionsForLoader = computed(() => {
	const versions = showSnapshots.value
		? tags.gameVersions.value
		: tags.gameVersions.value.filter((v) => v.version_type === 'release')

	if (selectedPlatform.value && selectedPlatform.value !== 'vanilla') {
		const manifest = manifestQuery.data.value?.gameVersions
		if (manifest) {
			const hasPlaceholder = manifest.some((x) => x.id === '${modrinth.gameVersion}')
			if (!hasPlaceholder) {
				const supportedVersions = new Set(
					manifest.filter((x) => x.loaders.length > 0).map((x) => x.id),
				)
				return versions.filter((v) => supportedVersions.has(v.version))
			}
		}
	}

	return versions
})

const hasSnapshots = computed(() =>
	gameVersionsForLoader.value.some((v) => v.version_type !== 'release'),
)

const gameVersionOptions = computed(() =>
	gameVersionsForLoader.value.map((v) => ({ value: v.version, label: v.version })),
)

const loaderVersionEntries = computed(() =>
	getLoaderVersionsForGameVersion(selectedPlatform.value, selectedGameVersion.value),
)

const loaderVersionOptions = computed(() =>
	loaderVersionEntries.value.map((v, index) => ({ value: index, label: v.id })),
)

const loaderVersionDisplayValue = computed(() => {
	const idx = selectedLoaderVersion.value
	return idx >= 0 && loaderVersionEntries.value[idx]
		? loaderVersionEntries.value[idx].id
		: formatMessage(commonMessages.selectVersionPlaceholder)
})

const formattedLoaderName = computed(() => {
	const name = selectedPlatform.value
	return name.charAt(0).toUpperCase() + name.slice(1)
})

const hasChanges = computed(() => {
	const currentLoader = server.value?.loader?.toLowerCase() ?? ''
	const currentGameVersion = server.value?.mc_version ?? ''
	const currentLoaderVersion = server.value?.loader_version ?? ''

	if (selectedPlatform.value !== currentLoader) return true
	if (selectedGameVersion.value !== currentGameVersion) return true
	if (
		selectedPlatform.value !== 'vanilla' &&
		loaderVersionEntries.value[selectedLoaderVersion.value]?.id !== currentLoaderVersion
	) {
		return true
	}
	return false
})

const isValid = computed(() => {
	if (!selectedGameVersion.value) return false
	if (selectedPlatform.value !== 'vanilla') {
		return selectedLoaderVersion.value >= 0 && loaderVersionEntries.value.length > 0
	}
	return true
})

watch(selectedPlatform, () => {
	selectedLoaderVersion.value = 0
})

watch(selectedGameVersion, () => {
	selectedLoaderVersion.value = 0
})

function toApiLoader(loader: string): Archon.Content.v1.Modloader {
	if (loader === 'neoforge') return 'neo_forge'
	return loader as Archon.Content.v1.Modloader
}

function cancelEditing() {
	selectedPlatform.value = server.value?.loader?.toLowerCase() ?? 'vanilla'
	selectedGameVersion.value = server.value?.mc_version ?? ''
	selectedLoaderVersion.value = 0
	isEditing.value = false
}

async function handleSave() {
	isSaving.value = true
	try {
		const loaderVersion = loaderVersionEntries.value[selectedLoaderVersion.value]?.id ?? ''

		const request: Archon.Content.v1.InstallWorldContent = {
			content_variant: 'bare',
			loader: toApiLoader(selectedPlatform.value),
			version: loaderVersion,
			game_version: selectedGameVersion.value || undefined,
			soft_override: true,
		}

		await client.archon.content_v1.installContent(serverId, worldId.value!, request)
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		isEditing.value = false
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : formatMessage(messages.failedToSaveSettings),
		})
	} finally {
		isSaving.value = false
	}
}

async function handleRepair() {
	if (modpack.value) {
		try {
			await client.archon.content_v1.installContent(serverId, worldId.value!, {
				content_variant: 'modpack',
				spec: {
					platform: 'modrinth',
					project_id: modpack.value.spec.project_id,
					version_id: modpack.value.spec.version_id,
				},
				soft_override: true,
			})
			await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		} catch (err) {
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : formatMessage(messages.failedToRepair),
			})
		}
	} else {
		const addons = addonsQuery.data.value
		const currentLoader = addons?.modloader ?? server.value?.loader
		const currentGameVersion = addons?.game_version ?? server.value?.mc_version
		const currentLoaderVersion = addons?.modloader_version ?? server.value?.loader_version
		if (!currentLoader || !currentGameVersion) return
		try {
			await client.archon.content_v1.installContent(serverId, worldId.value!, {
				content_variant: 'bare',
				loader: toApiLoader(currentLoader),
				version: currentLoaderVersion ?? '',
				game_version: currentGameVersion,
				soft_override: true,
			})
			await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		} catch (err) {
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : formatMessage(messages.failedToRepair),
			})
		}
	}
}

async function handleReinstallConfirm() {
	if (!modpack.value) return
	try {
		await client.archon.content_v1.installContent(serverId, worldId.value!, {
			content_variant: 'modpack',
			spec: {
				platform: 'modrinth',
				project_id: modpack.value.spec.project_id,
				version_id: modpack.value.spec.version_id,
			},
			soft_override: false,
		})
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		emit('reinstall')
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : formatMessage(messages.failedToReinstall),
		})
	}
}

async function handleUnlinkConfirm() {
	const previousData = addonsQuery.data.value
	if (previousData) {
		queryClient.setQueryData(['content', 'list', 'v1', serverId], {
			...previousData,
			modpack: null,
		})
	}

	try {
		await client.archon.content_v1.unlinkModpack(serverId, worldId.value!)
	} catch (err) {
		if (previousData) {
			queryClient.setQueryData(['content', 'list', 'v1', serverId], previousData)
		}
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : formatMessage(messages.failedToUnlink),
		})
	} finally {
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', serverId] }),
		])
	}
}

watch(
	() => server.value?.status,
	async (newStatus, oldStatus) => {
		if (oldStatus === 'installing' && newStatus === 'available') {
			selectedPlatform.value = server.value?.loader?.toLowerCase() ?? 'vanilla'
			selectedGameVersion.value = server.value?.mc_version ?? ''
			selectedLoaderVersion.value = 0
			isEditing.value = false

			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', serverId] }),
				queryClient.invalidateQueries({ queryKey: ['content', 'loader', 'versions'] }),
				queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
			])
		}
	},
)
</script>
