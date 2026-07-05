<template>
	<NewModal ref="modal" no-padding scrollable max-width="600px" width="600px" :on-hide="reset">
		<template #title>
			<span class="text-2xl font-semibold text-contrast">
				{{
					collectionName
						? formatMessage(messages.headerNamed, { name: collectionName })
						: formatMessage(messages.header)
				}}
			</span>
		</template>

		<template v-if="!lockedInstance">
			<div class="flex flex-col gap-2.5 p-6">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.instanceType) }}
				</span>
				<Chips
					v-model="tab"
					:items="['existing', 'new']"
					:format-label="formatTabLabel"
					:never-empty="true"
					:capitalize="false"
				/>
			</div>

			<div class="h-px bg-divider" />

			<div v-if="tab === 'existing'" class="flex flex-col gap-2.5 p-6">
				<span class="font-semibold text-contrast">
					{{ formatMessage(messages.selectInstance) }}
				</span>
				<div v-if="loadingInstances" class="flex items-center justify-center py-6">
					<LoadingIndicator />
				</div>
				<div
					v-else-if="instances.length === 0"
					class="flex items-center justify-center py-6 text-secondary"
				>
					{{ formatMessage(messages.noInstances) }}
				</div>
				<Combobox
					v-else
					v-model="selectedInstanceId"
					:options="instanceOptions"
					searchable
					:placeholder="formatMessage(messages.selectInstancePlaceholder)"
				/>
			</div>

			<div v-else class="flex flex-col gap-4 p-6">
				<div class="flex flex-col gap-2.5">
					<span class="font-semibold text-contrast">
						{{ formatMessage(messages.nameLabel) }}
					</span>
					<StyledInput
						v-model="newInstanceName"
						:placeholder="formatMessage(messages.namePlaceholder)"
					/>
				</div>
				<div class="flex flex-col gap-2.5">
					<span class="font-semibold text-contrast">
						{{ formatMessage(messages.loaderLabel) }}
					</span>
					<Chips
						v-model="newInstanceLoader"
						:items="NEW_INSTANCE_LOADERS"
						:format-label="formatLoaderLabel"
						:never-empty="true"
					/>
				</div>
				<div class="flex flex-col gap-2.5">
					<span class="font-semibold text-contrast">
						{{ formatMessage(commonMessages.gameVersionLabel) }}
					</span>
					<Combobox
						v-model="newInstanceGameVersion"
						:options="gameVersionOptions"
						searchable
						sync-with-selection
						:placeholder="formatMessage(messages.gameVersionPlaceholder)"
					>
						<template #dropdown-footer>
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
			</div>

			<div class="h-px bg-divider" />
		</template>

		<div class="flex flex-col gap-2.5 p-6">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast">
					{{
						formatMessage(messages.projectsLabel, {
							selected: checkedIds.size,
							total: projects.length,
						})
					}}
				</span>
				<ButtonStyled v-if="selectableProjects.length > 0" type="transparent">
					<button @click="toggleAll">
						{{
							checkedIds.size > 0
								? formatMessage(messages.deselectAll)
								: formatMessage(messages.selectAll)
						}}
					</button>
				</ButtonStyled>
			</div>

			<div class="flex max-h-[320px] flex-col gap-0.5 overflow-y-auto rounded-xl bg-surface-2 p-2">
				<div v-if="checkingTarget" class="flex items-center justify-center py-6">
					<LoadingIndicator />
				</div>
				<template v-else>
					<label
						v-for="project in projects"
						:key="project.id"
						class="flex items-center gap-2.5 rounded-lg px-2 py-1.5"
						:class="
							projectStatus(project) === 'ok' ? 'cursor-pointer hover:bg-surface-3' : 'opacity-60'
						"
					>
						<Checkbox
							:model-value="checkedIds.has(project.id)"
							:disabled="projectStatus(project) !== 'ok' || installing"
							@update:model-value="(value) => setChecked(project.id, value)"
						/>
						<Avatar :src="project.icon_url" size="2rem" rounded="md" />
						<span class="min-w-0 flex-1 truncate font-semibold text-contrast">
							{{ project.title }}
						</span>
						<span
							v-if="projectStatus(project) === 'installed'"
							class="flex shrink-0 items-center gap-1 text-sm text-secondary"
						>
							<CheckIcon class="size-4" aria-hidden="true" />
							{{ formatMessage(messages.alreadyInstalled) }}
						</span>
						<span
							v-else-if="projectStatus(project) === 'incompatible'"
							v-tooltip="formatMessage(messages.incompatibleTooltip)"
							class="flex shrink-0 items-center gap-1 text-sm text-orange"
						>
							<TriangleAlertIcon class="size-4" aria-hidden="true" />
							{{ formatMessage(messages.incompatible) }}
						</span>
						<span
							v-else-if="projectStatus(project) === 'unsupported'"
							class="flex shrink-0 items-center gap-1 text-sm text-secondary"
						>
							<XIcon class="size-4" aria-hidden="true" />
							{{ formatMessage(messages.unsupported) }}
						</span>
					</label>
				</template>
			</div>

			<div v-if="installing" class="flex items-center gap-2 text-secondary">
				<SpinnerIcon class="size-4 animate-spin" aria-hidden="true" />
				{{
					formatMessage(messages.installingProgress, {
						current: progressCurrent,
						total: progressTotal,
					})
				}}
			</div>
		</div>

		<template #actions>
			<div class="flex items-center justify-end gap-2">
				<ButtonStyled type="outlined">
					<button :disabled="installing" @click="modal?.hide()">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="!canInstall" @click="handleInstall">
						<SpinnerIcon v-if="installing" class="animate-spin" aria-hidden="true" />
						<DownloadIcon v-else aria-hidden="true" />
						{{
							installing
								? formatMessage(commonMessages.installingLabel)
								: formatMessage(messages.installButton, { count: checkedIds.size })
						}}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	DownloadIcon,
	EyeIcon,
	EyeOffIcon,
	SpinnerIcon,
	TriangleAlertIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	Checkbox,
	Chips,
	Combobox,
	type ComboboxOption,
	commonMessages,
	defineMessages,
	formatLoaderLabel,
	injectNotificationManager,
	LoadingIndicator,
	NewModal,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import {
	install_create_instance,
	installJobInstanceId,
	wait_for_install_job,
} from '@/helpers/install'
import {
	get_installed_project_ids,
	install_project_with_dependencies,
	list,
} from '@/helpers/instance'
import { get_game_versions } from '@/helpers/tags.js'
import type { GameInstance, InstanceLoader } from '@/helpers/types'

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()

const messages = defineMessages({
	header: {
		id: 'app.collection-install.header',
		defaultMessage: 'Install collection',
	},
	headerNamed: {
		id: 'app.collection-install.header-named',
		defaultMessage: 'Install {name}',
	},
	instanceType: {
		id: 'app.collection-install.instance-type',
		defaultMessage: 'Instance type',
	},
	existingTab: {
		id: 'app.collection-install.existing-tab',
		defaultMessage: 'Existing instance',
	},
	newTab: {
		id: 'app.collection-install.new-tab',
		defaultMessage: 'New instance',
	},
	selectInstance: {
		id: 'app.collection-install.select-instance',
		defaultMessage: 'Instance',
	},
	selectInstancePlaceholder: {
		id: 'app.collection-install.select-instance-placeholder',
		defaultMessage: 'Select an instance',
	},
	noInstances: {
		id: 'app.collection-install.no-instances',
		defaultMessage: 'No instances found. Create a new instance instead.',
	},
	nameLabel: {
		id: 'app.collection-install.name-label',
		defaultMessage: 'Name',
	},
	namePlaceholder: {
		id: 'app.collection-install.name-placeholder',
		defaultMessage: 'Enter instance name',
	},
	loaderLabel: {
		id: 'app.collection-install.loader-label',
		defaultMessage: 'Loader',
	},
	gameVersionPlaceholder: {
		id: 'app.collection-install.game-version-placeholder',
		defaultMessage: 'Select game version',
	},
	projectsLabel: {
		id: 'app.collection-install.projects-label',
		defaultMessage: '{selected} of {total} projects selected',
	},
	selectAll: {
		id: 'app.collection-install.select-all',
		defaultMessage: 'Select all',
	},
	deselectAll: {
		id: 'app.collection-install.deselect-all',
		defaultMessage: 'Deselect all',
	},
	alreadyInstalled: {
		id: 'app.collection-install.already-installed',
		defaultMessage: 'Installed',
	},
	incompatible: {
		id: 'app.collection-install.incompatible',
		defaultMessage: 'Incompatible',
	},
	incompatibleTooltip: {
		id: 'app.collection-install.incompatible-tooltip',
		defaultMessage:
			'This project has no version compatible with the selected loader and game version.',
	},
	unsupported: {
		id: 'app.collection-install.unsupported',
		defaultMessage: 'Not supported',
	},
	installButton: {
		id: 'app.collection-install.install-button',
		defaultMessage: 'Install {count, plural, one {# project} other {# projects}}',
	},
	installingProgress: {
		id: 'app.collection-install.installing-progress',
		defaultMessage: 'Installing {current, number} of {total, number}...',
	},
	installSuccess: {
		id: 'app.collection-install.success',
		defaultMessage: 'Collection installed',
	},
	installSuccessText: {
		id: 'app.collection-install.success-text',
		defaultMessage:
			'{count, plural, one {# project} other {# projects}} added to {instance}. Missing dependencies were installed automatically.',
	},
	installPartialText: {
		id: 'app.collection-install.partial-text',
		defaultMessage:
			'{count, plural, one {# project} other {# projects}} added to {instance}, {failed} failed.',
	},
	installFailed: {
		id: 'app.collection-install.failed',
		defaultMessage: 'Failed to install collection',
	},
})

type ProjectStatus = 'ok' | 'installed' | 'incompatible' | 'unsupported'

const RESOLVABLE_PROJECT_TYPES = new Set(['mod', 'plugin', 'datapack', 'resourcepack', 'shader'])
const NEW_INSTANCE_LOADERS = ['vanilla', 'fabric', 'quilt', 'neoforge', 'forge']

const emit = defineEmits<{
	installed: [instanceId: string]
}>()

const modal = ref<InstanceType<typeof NewModal>>()

const projects = ref<Labrinth.Projects.v2.Project[]>([])
const collectionName = ref<string | null>(null)
const lockedInstance = ref<GameInstance | null>(null)

const tab = ref<'existing' | 'new'>('existing')
const tabLabels: Record<'existing' | 'new', () => string> = {
	existing: () => formatMessage(messages.existingTab),
	new: () => formatMessage(messages.newTab),
}
const formatTabLabel = (item: 'existing' | 'new') => tabLabels[item]()

const instances = ref<GameInstance[]>([])
const loadingInstances = ref(false)
const selectedInstanceId = ref<string | null>(null)

const newInstanceName = ref('')
const newInstanceLoader = ref<string>('fabric')
const newInstanceGameVersion = ref<string | null>(null)
const allGameVersions = ref<{ version: string; version_type: string }[]>([])
const showSnapshots = ref(false)

const installedProjectIds = ref<Set<string>>(new Set())
const checkingTarget = ref(false)
const checkedIds = ref<Set<string>>(new Set())

const installing = ref(false)
const progressCurrent = ref(0)
const progressTotal = ref(0)

const instanceOptions = computed<ComboboxOption<string>[]>(() =>
	instances.value.map((instance) => ({
		value: instance.id,
		label: `${instance.name} (${formatLoaderLabel(instance.loader)} ${instance.game_version})`,
	})),
)

const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	const versions = showSnapshots.value
		? allGameVersions.value
		: allGameVersions.value.filter((v) => v.version_type === 'release')
	return versions.map((v) => ({ value: v.version, label: v.version }))
})

const targetLoader = computed(() => {
	if (lockedInstance.value) return lockedInstance.value.loader
	if (tab.value === 'new') return newInstanceLoader.value
	return (
		instances.value.find((instance) => instance.id === selectedInstanceId.value)?.loader ?? null
	)
})

const targetGameVersion = computed(() => {
	if (lockedInstance.value) return lockedInstance.value.game_version
	if (tab.value === 'new') return newInstanceGameVersion.value
	return (
		instances.value.find((instance) => instance.id === selectedInstanceId.value)?.game_version ??
		null
	)
})

const hasTarget = computed(() => !!targetLoader.value && !!targetGameVersion.value)

function isProjectCompatible(project: Labrinth.Projects.v2.Project) {
	if (!targetLoader.value || !targetGameVersion.value) return false

	const gameVersions = project.game_versions ?? []
	if (!gameVersions.includes(targetGameVersion.value)) return false

	if (project.project_type === 'mod' || project.project_type === 'plugin') {
		const loaders = project.loaders ?? []
		return loaders.includes(targetLoader.value) || loaders.includes('datapack')
	}

	return true
}

function projectStatus(project: Labrinth.Projects.v2.Project): ProjectStatus {
	if (!RESOLVABLE_PROJECT_TYPES.has(project.project_type)) return 'unsupported'
	if (installedProjectIds.value.has(project.id)) return 'installed'
	if (!hasTarget.value || !isProjectCompatible(project)) return 'incompatible'
	return 'ok'
}

const selectableProjects = computed(() =>
	projects.value.filter((project) => projectStatus(project) === 'ok'),
)

const canInstall = computed(() => {
	if (installing.value || checkedIds.value.size === 0) return false
	if (lockedInstance.value) return true
	if (tab.value === 'existing') return !!selectedInstanceId.value
	return !!newInstanceName.value && !!newInstanceLoader.value && !!newInstanceGameVersion.value
})

function setChecked(projectId: string, value: boolean) {
	const next = new Set(checkedIds.value)
	if (value) next.add(projectId)
	else next.delete(projectId)
	checkedIds.value = next
}

function toggleAll() {
	if (checkedIds.value.size > 0) {
		checkedIds.value = new Set()
	} else {
		checkedIds.value = new Set(selectableProjects.value.map((project) => project.id))
	}
}

function selectAllEligible() {
	checkedIds.value = new Set(selectableProjects.value.map((project) => project.id))
}

async function refreshInstalledProjectIds(instanceId: string | null) {
	checkingTarget.value = true
	try {
		installedProjectIds.value = instanceId
			? new Set(await get_installed_project_ids(instanceId))
			: new Set()
	} catch (err) {
		installedProjectIds.value = new Set()
		handleError(err as Error)
	} finally {
		checkingTarget.value = false
	}
}

watch([selectedInstanceId, tab], async ([instanceId, newTab]) => {
	if (lockedInstance.value || installing.value) return
	await refreshInstalledProjectIds(newTab === 'existing' ? instanceId : null)
	selectAllEligible()
})

watch([newInstanceLoader, newInstanceGameVersion], () => {
	if (lockedInstance.value || installing.value || tab.value !== 'new') return
	selectAllEligible()
})

async function show(
	projectList: Labrinth.Projects.v2.Project[],
	options?: { instance?: GameInstance | null; collectionName?: string | null },
) {
	projects.value = projectList
	collectionName.value = options?.collectionName ?? null
	lockedInstance.value = options?.instance ?? null
	tab.value = 'existing'
	selectedInstanceId.value = null
	newInstanceName.value = collectionName.value ?? ''
	newInstanceLoader.value = 'fabric'
	newInstanceGameVersion.value = null
	showSnapshots.value = false
	installedProjectIds.value = new Set()
	checkedIds.value = new Set()
	installing.value = false
	progressCurrent.value = 0
	progressTotal.value = 0

	modal.value?.show()

	if (lockedInstance.value) {
		await refreshInstalledProjectIds(lockedInstance.value.id)
		selectAllEligible()
	} else {
		loadingInstances.value = true
		try {
			instances.value = await list()
			const firstCompatible = instances.value.find((instance) =>
				projectList.some(
					(project) =>
						(project.game_versions ?? []).includes(instance.game_version) &&
						(project.project_type !== 'mod' || (project.loaders ?? []).includes(instance.loader)),
				),
			)
			selectedInstanceId.value = firstCompatible?.id ?? instances.value[0]?.id ?? null
			if (instances.value.length === 0) {
				tab.value = 'new'
			}
		} catch (err) {
			handleError(err as Error)
		} finally {
			loadingInstances.value = false
		}
	}

	if (allGameVersions.value.length === 0) {
		get_game_versions()
			.then((versions: { version: string; version_type: string }[]) => {
				allGameVersions.value = versions
				if (!newInstanceGameVersion.value) {
					newInstanceGameVersion.value =
						versions.find((v) => v.version_type === 'release')?.version ?? null
				}
			})
			.catch(() => {})
	} else if (!newInstanceGameVersion.value) {
		newInstanceGameVersion.value =
			allGameVersions.value.find((v) => v.version_type === 'release')?.version ?? null
	}
}

function reset() {
	if (!installing.value) {
		projects.value = []
		checkedIds.value = new Set()
	}
}

function resolveContentType(projectType: string): Labrinth.Content.v3.ContentType {
	return RESOLVABLE_PROJECT_TYPES.has(projectType)
		? (projectType as Labrinth.Content.v3.ContentType)
		: 'mod'
}

async function resolveTargetInstanceId(): Promise<string> {
	if (lockedInstance.value) return lockedInstance.value.id
	if (tab.value === 'existing') {
		if (!selectedInstanceId.value) throw new Error('No instance selected')
		return selectedInstanceId.value
	}

	const job = await install_create_instance({
		name: newInstanceName.value,
		gameVersion: newInstanceGameVersion.value!,
		loader: newInstanceLoader.value as InstanceLoader,
		loaderVersion: 'latest',
		iconPath: null,
	})
	const finished = await wait_for_install_job(job.job_id)
	const instanceId = installJobInstanceId(finished)
	if (!instanceId) throw new Error('Failed to create instance')
	return instanceId
}

async function handleInstall() {
	if (!canInstall.value) return

	const selectedProjects = projects.value.filter(
		(project) => checkedIds.value.has(project.id) && projectStatus(project) === 'ok',
	)
	if (selectedProjects.length === 0) return

	installing.value = true
	progressCurrent.value = 0
	progressTotal.value = selectedProjects.length

	let instanceId: string
	try {
		instanceId = await resolveTargetInstanceId()
	} catch (err) {
		installing.value = false
		handleError(err as Error)
		return
	}

	const instanceName = lockedInstance.value
		? lockedInstance.value.name
		: tab.value === 'existing'
			? (instances.value.find((instance) => instance.id === instanceId)?.name ?? instanceId)
			: newInstanceName.value

	let added = 0
	const failed: string[] = []

	for (const project of selectedProjects) {
		progressCurrent.value += 1
		try {
			await install_project_with_dependencies(instanceId, {
				project_id: project.id,
				content_type: resolveContentType(project.project_type),
			})
			added += 1
		} catch (err) {
			console.error(`Failed to install ${project.title}:`, err)
			failed.push(project.title)
		}
	}

	installing.value = false

	if (added > 0) {
		addNotification({
			type: failed.length > 0 ? 'warning' : 'success',
			title: formatMessage(messages.installSuccess),
			text:
				failed.length > 0
					? formatMessage(messages.installPartialText, {
							count: added,
							instance: instanceName,
							failed: failed.length,
						})
					: formatMessage(messages.installSuccessText, {
							count: added,
							instance: instanceName,
						}),
		})
		emit('installed', instanceId)
		modal.value?.hide()
	} else {
		addNotification({
			type: 'error',
			title: formatMessage(messages.installFailed),
			text: failed.join(', '),
		})
	}
}

defineExpose({ show })
</script>
