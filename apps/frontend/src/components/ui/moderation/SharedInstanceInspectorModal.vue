<script setup lang="ts">
import type { SharedInstances } from '@modrinth/api-client'
import {
	BoxesIcon,
	CopyIcon,
	DownloadIcon,
	HistoryIcon,
	LinkIcon,
	LoaderCircleIcon,
	LockIcon,
	LockOpenIcon,
	SearchIcon,
	SortAscIcon,
	UserIcon,
	UserPlusIcon,
	UsersIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	ButtonLink,
	Combobox,
	type ComboboxOption,
	ConfirmLeaveModal,
	ContentCardTable,
	type ContentCardTableItem,
	type ContentItem,
	CopyCode,
	FilterPills,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	NewModal,
	useFormatBytes,
	useFormatDateTime,
	useRelativeTime,
	useScrollIndicator,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import {
	type SharedInstanceModerationUser,
	sharedInstanceQueryOptions,
} from '~/composables/queries/shared-instance'
import {
	formatSharedInstanceLoader,
	getRequestErrorMessage,
	sharedInstanceInspectorPath,
	useSharedInstanceModeration,
} from '~/composables/shared-instance-moderation'

import SharedInstanceQuarantineModal from './SharedInstanceQuarantineModal.vue'

type SortMode = 'uploaded' | 'name'
type RowKind = 'uploaded' | 'config' | 'modrinth'
type InspectorRow = {
	kind: RowKind
	typeFilter: string
	searchText: string
	sortName: string
	tableItem: ContentCardTableItem
	fileName: string
	fileType: string
}

const CONFIG_FILE_TYPES = new Set(['config', 'configs'])
const TYPE_LABELS: Record<string, string> = {
	mod: 'Mods',
	resourcepack: 'Resource Packs',
	shader: 'Shaders',
	datapack: 'Data Packs',
	plugin: 'Plugins',
	modpack: 'Modpacks',
}

const emit = defineEmits<{
	hide: []
}>()

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const { pending: actionPending, liftQuarantine, deleteFile } = useSharedInstanceModeration()
const formatBytes = useFormatBytes()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const quarantineModal = ref<InstanceType<typeof SharedInstanceQuarantineModal> | null>(null)
const deleteModal = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
const instanceId = ref<string | null>(null)
const selectedVersion = ref<number | null>(null)
const reportedVersions = ref<number[]>([])
const searchQuery = ref('')
const sortMode = ref<SortMode>('uploaded')
const selectedFilters = ref<string[]>([])
const pendingDelete = ref<InspectorRow | null>(null)
const tableScroll = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade } = useScrollIndicator(tableScroll)

const {
	data: details,
	error: detailsError,
	isPending: detailsPending,
	refetch: refetchDetails,
} = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.moderationContext(instanceId.value ?? '', client, queryClient),
		enabled: instanceId.value !== null,
	})),
)

const activeVersion = computed(() => selectedVersion.value ?? details.value?.versions[0]?.version)
const versionQueryEnabled = computed(
	() => instanceId.value !== null && activeVersion.value !== undefined,
)

const { data: rawVersion } = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.version(instanceId.value ?? '', activeVersion.value ?? 0, client),
		enabled: versionQueryEnabled.value,
	})),
)

const {
	data: content,
	error: contentError,
	isPending: contentPending,
} = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.versionContent(
			instanceId.value ?? '',
			activeVersion.value ?? 0,
			client,
			queryClient,
		),
		enabled: versionQueryEnabled.value,
	})),
)

watch(activeVersion, () => {
	searchQuery.value = ''
	selectedFilters.value = []
})

const instanceSummary = computed(() => ({
	id: instanceId.value ?? '',
	name: details.value?.name ?? 'Shared instance',
	icon_url: details.value?.icon_url,
}))
const isInstanceQuarantined = computed(() => details.value?.quarantine_details?.scope === 'instance')

const versionOptions = computed<ComboboxOption<number>[]>(() =>
	(details.value?.versions ?? []).map((version) => ({
		value: version.version,
		label: `Version ${version.version}${reportedVersions.value.includes(version.version) ? ' · Reported' : ''}`,
		subLabel: version.game_version
			? `Minecraft ${version.game_version} · ${formatSharedInstanceLoader(version)}`
			: undefined,
	})),
)

const externalFilesByKey = computed(() => {
	const files = new Map<string, SharedInstances.Instances.v1.ExternalFile>()
	for (const file of rawVersion.value?.external_files ?? []) {
		files.set(`${file.file_type}:${file.file_name}`, file)
	}
	return files
})

const rows = computed<InspectorRow[]>(() =>
	(content.value ?? []).map((item) => (item.external ? externalRow(item) : modrinthRow(item))),
)

function externalRow(item: ContentItem): InspectorRow {
	const fileType = item.project_type
	const file = externalFilesByKey.value.get(`${fileType}:${item.file_name}`)
	const isConfig = CONFIG_FILE_TYPES.has(fileType)
	const sha512 = file?.sha512
	const downloadUrl = file?.url ?? item.external_url ?? null

	return {
		kind: isConfig ? 'config' : 'uploaded',
		typeFilter: normalizeType(fileType),
		searchText: item.file_name.toLowerCase(),
		sortName: item.file_name.toLowerCase(),
		fileName: item.file_name,
		fileType,
		tableItem: {
			id: item.id,
			projectType: isConfig ? undefined : fileType,
			project: { id: item.id, slug: item.file_name, title: item.file_name, icon_url: undefined },
			version: {
				id: item.id,
				version_number: item.size != null ? formatBytes(item.size) : 'Unknown size',
				file_name: sha512 ? `sha512 ${sha512.slice(0, 4)}…${sha512.slice(-4)}` : '',
			},
			external: true,
			externalFile: !isConfig,
			overrideFile: isConfig,
			overflowOptions: [
				...(downloadUrl
					? [
							{
								id: 'download',
								type: 'link' as const,
								label: 'Download',
								icon: DownloadIcon,
								href: downloadUrl,
								target: '_blank',
								rel: 'noopener noreferrer',
							},
						]
					: []),
				...(sha512
					? [
							{
								id: 'copy-sha512',
								label: 'Copy SHA-512',
								icon: CopyIcon,
								action: () => copyText(sha512, 'SHA-512 hash copied'),
							},
						]
					: []),
				...(downloadUrl && item.file_name.endsWith('.jar')
					? [
							{
								id: 'slicer',
								type: 'link' as const,
								label: 'View in Slicer',
								icon: SearchIcon,
								href: `https://slicer.run/?url=${encodeURIComponent(downloadUrl)}`,
								target: '_blank',
								rel: 'noopener noreferrer',
							},
						]
					: []),
			],
		},
	}
}

function modrinthRow(item: ContentItem): InspectorRow {
	const title = item.project?.title ?? item.file_name
	const versionId = item.version?.id

	return {
		kind: 'modrinth',
		typeFilter: normalizeType(item.project_type),
		searchText: `${title} ${item.file_name}`.toLowerCase(),
		sortName: title.toLowerCase(),
		fileName: item.file_name,
		fileType: item.project_type,
		tableItem: {
			id: item.id,
			projectType: item.project_type,
			project: item.project ?? {
				id: item.id,
				slug: item.file_name,
				title: item.file_name,
				icon_url: undefined,
			},
			projectLink: item.project?.id ? `/project/${item.project.id}` : undefined,
			version: item.version,
			owner: item.owner
				? {
						...item.owner,
						link: `/${item.owner.type}/${encodeURIComponent(item.owner.id)}`,
					}
				: undefined,
			versionLink:
				item.project?.id && versionId
					? `/project/${item.project.id}/version/${versionId}`
					: undefined,
			source: item.source
				? {
						...item.source,
						label: 'Provided by',
						link: `/project/${item.source.project.slug ?? item.source.project.id}`,
					}
				: undefined,
			hideDelete: true,
			overflowOptions: versionId
				? [
						{
							id: 'copy-version-id',
							label: 'Copy version ID',
							icon: CopyIcon,
							action: () => copyText(versionId, 'Version ID copied'),
						},
					]
				: [],
		},
	}
}

function normalizeType(type: string) {
	if (CONFIG_FILE_TYPES.has(type)) return 'config'
	if (type === 'shaderpack') return 'shader'
	return type
}

const filterOptions = computed(() => {
	const types = new Set(rows.value.map((row) => row.typeFilter))
	return [
		...(rows.value.some((row) => row.kind === 'uploaded')
			? [{ id: 'uploaded', label: 'Uploaded' }]
			: []),
		...Object.entries(TYPE_LABELS)
			.filter(([type]) => types.has(type))
			.map(([type, label]) => ({ id: type, label })),
		...(types.has('config') ? [{ id: 'config', label: 'Config' }] : []),
	]
})

const sortOptions: ComboboxOption<SortMode>[] = [
	{ value: 'uploaded', label: 'Uploaded first' },
	{ value: 'name', label: 'Name' },
]

const KIND_ORDER: Record<RowKind, number> = { uploaded: 0, config: 1, modrinth: 2 }

const visibleRows = computed(() => {
	const query = searchQuery.value.trim().toLowerCase()
	const filters = selectedFilters.value
	return rows.value
		.filter((row) => !query || row.searchText.includes(query))
		.filter(
			(row) =>
				filters.length === 0 ||
				filters.includes(row.typeFilter) ||
				(filters.includes('uploaded') && row.kind === 'uploaded'),
		)
		.sort(
			(first, second) =>
				(sortMode.value === 'uploaded' ? KIND_ORDER[first.kind] - KIND_ORDER[second.kind] : 0) ||
				first.sortName.localeCompare(second.sortName),
		)
})
const tableItems = computed(() => visibleRows.value.map((row) => row.tableItem))

const memberRows = computed<SharedInstanceModerationUser[]>(() =>
	details.value ? [details.value.owner, ...details.value.members] : [],
)

function memberSummary(member: SharedInstanceModerationUser) {
	const played = member.last_played
		? `Played ${formatRelativeTime(member.last_played)}`
		: 'Never played'
	return member.joined_at ? `${played} · joined ${formatRelativeTime(member.joined_at)}` : played
}

function memberMethod(member: SharedInstanceModerationUser) {
	if (member.join_type === 'owner') return { icon: UsersIcon, label: 'Owner' }
	if (member.join_type === 'link') return { icon: LinkIcon, label: 'Share link' }
	return { icon: UserPlusIcon, label: 'Direct invite' }
}

async function copyText(text: string, title: string) {
	await navigator.clipboard.writeText(text)
	addNotification({ type: 'success', title, text })
}

async function handleDelete(id: string, event: MouseEvent) {
	const row = visibleRows.value.find((candidate) => candidate.tableItem.id === id)
	if (!row || row.kind === 'modrinth' || activeVersion.value === undefined) return

	pendingDelete.value = row
	if (!event.shiftKey && !(await deleteModal.value?.prompt())) return
	await deleteFile(instanceSummary.value, activeVersion.value, row.fileName, row.fileType)
}

function copyInspectorLink() {
	if (!details.value) return
	const path = sharedInstanceInspectorPath(
		details.value.owner.id,
		details.value.id,
		activeVersion.value,
	)
	void copyText(`${window.location.origin}${path}`, 'Inspector link copied')
}

function show(id: string, options: { version?: number; reportedVersions?: number[] } = {}) {
	instanceId.value = id
	selectedVersion.value = options.version ?? null
	reportedVersions.value = options.reportedVersions ?? []
	searchQuery.value = ''
	sortMode.value = 'uploaded'
	selectedFilters.value = []
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modal"
		no-padding
		max-width="1180px"
		width="calc(100vw - 2rem)"
		:on-hide="() => emit('hide')"
	>
		<template #title>
			<div class="flex min-w-0 flex-wrap items-center gap-3">
				<Avatar
					:src="details?.icon_url"
					:alt="instanceSummary.name"
					:tint-by="instanceSummary.id"
					size="3rem"
					no-shadow
					pad-transparent-corners
				/>
				<span class="truncate text-2xl font-semibold text-contrast">{{ instanceSummary.name }}</span>
				<CopyCode v-if="instanceId" v-tooltip="'Copy instance ID'" :text="instanceId" />
			</div>
		</template>
		<template #header-actions>
			<div v-if="details" class="flex items-center gap-2">
				<div v-if="versionOptions.length" class="w-56 shrink-0">
					<Combobox
						:model-value="activeVersion"
						:options="versionOptions"
						dropdown-min-width="18rem"
						dropdown-align="right"
						@update:model-value="(value) => (selectedVersion = value as number)"
					/>
				</div>
				<IconButton
					v-tooltip="'Copy inspector link'"
					label="Copy inspector link"
					@click="copyInspectorLink"
				>
					<LinkIcon />
				</IconButton>
				<Button
					v-if="isInstanceQuarantined"
					type="outlined"
					color="orange"
					:loading="actionPending"
					@click="liftQuarantine(instanceSummary)"
				>
					<LockOpenIcon />
					Lift quarantine
				</Button>
				<Button v-else type="colored" color="orange" @click="quarantineModal?.show()">
					<LockIcon />
					Quarantine
				</Button>
			</div>
		</template>

		<div
			v-if="detailsPending"
			class="flex min-h-64 items-center justify-center gap-2 p-6 text-secondary"
		>
			<LoaderCircleIcon class="size-6 animate-spin" />
			Loading shared instance…
		</div>
		<div v-else-if="detailsError" class="p-6">
			<Admonition
				type="critical"
				header="Failed to load shared instance"
				:body="getRequestErrorMessage(detailsError, 'The shared instance could not be loaded.')"
				show-actions-underneath
			>
				<template #actions>
					<Button @click="refetchDetails()">Try again</Button>
				</template>
			</Admonition>
		</div>
		<div
			v-else-if="details"
			class="flex h-[min(720px,calc(95vh-8rem))] flex-col overflow-hidden lg:flex-row"
		>
			<section aria-label="Files" class="flex min-h-0 min-w-0 flex-1 flex-col">
				<div
					class="flex shrink-0 flex-col gap-4 border-0 border-b border-solid border-surface-4 px-6 py-4"
				>
					<Input
						v-model="searchQuery"
						:icon="SearchIcon"
						type="search"
						:placeholder="`Search ${rows.length} ${rows.length === 1 ? 'file' : 'files'}`"
						:aria-label="`Search ${rows.length} ${rows.length === 1 ? 'file' : 'files'}`"
						clearable
					/>
					<div class="flex flex-wrap items-center gap-2">
						<div class="w-44 shrink-0">
							<Combobox
								v-model="sortMode"
								:options="sortOptions"
								trigger-size="sm"
								aria-label="Sort files"
							>
								<template #prefix>
									<SortAscIcon class="size-4 shrink-0" />
								</template>
							</Combobox>
						</div>
						<template v-if="filterOptions.length > 1">
							<div class="h-6 w-px shrink-0 bg-surface-5" />
							<FilterPills v-model="selectedFilters" :options="filterOptions" />
						</template>
					</div>
				</div>

				<div v-if="!versionQueryEnabled" class="p-8 text-center text-secondary">
					No versions have been published.
				</div>
				<div
					v-else-if="contentPending"
					class="flex flex-1 items-center justify-center gap-2 p-8 text-secondary"
				>
					<LoaderCircleIcon class="size-6 animate-spin" />
					Loading files…
				</div>
				<div v-else-if="contentError" class="p-6">
					<Admonition
						type="critical"
						header="Failed to load files"
						:body="getRequestErrorMessage(contentError, 'The files for this version could not be loaded.')"
					/>
				</div>
				<div v-else-if="!rows.length" class="p-8 text-center text-secondary">
					This version has no files.
				</div>
				<div v-else-if="!visibleRows.length" class="p-8 text-center text-secondary">
					No files match your search.
				</div>
				<div v-else class="relative min-h-0 flex-1">
					<div ref="tableScroll" class="@container h-full overflow-y-auto px-6 pb-6">
						<ContentCardTable
							class="mt-4"
							:items="tableItems"
							:virtualized="false"
							@delete="handleDelete"
						/>
					</div>
					<div
						v-if="showTopFade"
						aria-hidden="true"
						class="pointer-events-none absolute inset-x-0 top-0 z-10 h-6 bg-gradient-to-b from-surface-3 to-transparent"
					/>
					<div
						v-if="showBottomFade"
						aria-hidden="true"
						class="pointer-events-none absolute inset-x-0 bottom-0 z-10 h-6 bg-gradient-to-t from-surface-3 to-transparent"
					/>
				</div>
			</section>

			<aside
				aria-label="Instance details"
				class="flex max-h-[45%] shrink-0 flex-col gap-6 overflow-y-auto border-0 border-t border-solid border-surface-4 bg-surface-2 p-6 lg:max-h-none lg:w-[22rem] lg:border-l lg:border-t-0"
			>
				<div class="flex flex-col gap-3">
					<span class="flex items-center gap-2 font-semibold text-contrast">
						<UserIcon class="size-5 text-secondary" />
						Owner
					</span>
					<div class="flex flex-wrap items-center gap-2">
						<NuxtLink
							:to="`/user/${details.owner.id}`"
							target="_blank"
							class="inline-flex min-w-0 items-center gap-2 font-semibold text-contrast hover:underline"
						>
							<Avatar
								:src="details.owner.avatar_url"
								:alt="details.owner.username"
								:tint-by="details.owner.id"
								size="2rem"
								circle
								no-shadow
							/>
							<span class="truncate">{{ details.owner.username }}</span>
						</NuxtLink>
						<CopyCode v-tooltip="'Copy user ID'" :text="details.owner.id" />
					</div>
					<ButtonLink :to="`/admin/shared-instances/${details.owner.id}`" size="sm" class="w-fit">
						<BoxesIcon />
						View all instances
					</ButtonLink>
				</div>

				<div class="flex flex-col gap-3">
					<span class="flex items-center gap-2 font-semibold text-contrast">
						<UsersIcon class="size-5 text-secondary" />
						Members
					</span>
					<div class="overflow-hidden rounded-2xl border border-solid border-surface-4">
						<div
							v-for="(member, index) in memberRows"
							:key="member.id"
							class="flex min-h-14 items-center gap-2 px-4 py-2"
							:class="[
								index % 2 === 0 ? 'bg-surface-2' : 'bg-surface-1',
								{ 'border-0 border-t border-solid border-surface-4': index > 0 },
							]"
						>
							<Avatar
								:src="member.avatar_url"
								:alt="member.username"
								:tint-by="member.id"
								size="1.5rem"
								circle
								no-shadow
							/>
							<span class="flex min-w-0 flex-1 flex-col">
								<NuxtLink
									:to="`/user/${member.id}`"
									target="_blank"
									class="truncate font-medium leading-5 text-primary hover:underline"
								>
									{{ member.username }}
								</NuxtLink>
								<span
									v-tooltip="member.last_played ? formatDateTime(member.last_played) : undefined"
									class="text-xs text-secondary"
								>
									{{ memberSummary(member) }}
								</span>
							</span>
							<span class="inline-flex shrink-0 items-center gap-1.5 text-sm text-secondary">
								<component :is="memberMethod(member).icon" class="size-4" />
								{{ memberMethod(member).label }}
							</span>
						</div>
					</div>
				</div>

				<div class="flex flex-col gap-3">
					<span class="flex items-center gap-2 font-semibold text-contrast">
						<HistoryIcon class="size-5 text-secondary" />
						Version history
					</span>
					<div v-if="details.versions.length" class="flex max-h-80 flex-col gap-2 overflow-y-auto">
						<button
							v-for="version in details.versions"
							:key="version.version"
							type="button"
							class="flex cursor-pointer items-center gap-2.5 rounded-xl border-0 bg-surface-3 p-3 text-left"
							:class="
								version.version === activeVersion
									? 'shadow-[inset_0_0_0_1px_var(--color-brand)]'
									: 'hover:bg-surface-4'
							"
							:aria-pressed="version.version === activeVersion"
							@click="selectedVersion = version.version"
						>
							<span class="backed-svg shrink-0">
								<VersionIcon />
							</span>
							<span class="flex min-w-0 flex-col">
								<span class="flex flex-wrap items-center gap-2">
									<span class="font-semibold text-contrast">Version {{ version.version }}</span>
									<span
										v-if="reportedVersions.includes(version.version)"
										class="bg-orange-highlight rounded-full px-2 py-0.5 text-xs font-semibold text-orange"
									>
										Reported
									</span>
								</span>
								<span v-if="version.game_version" class="truncate text-sm text-secondary">
									Minecraft {{ version.game_version }} · {{ formatSharedInstanceLoader(version) }}
								</span>
							</span>
						</button>
					</div>
					<span v-else class="text-sm text-secondary">No versions have been published.</span>
				</div>
			</aside>
		</div>

		<SharedInstanceQuarantineModal ref="quarantineModal" :instance="instanceSummary" />
		<ConfirmLeaveModal
			ref="deleteModal"
			title="Delete file?"
			:header="`Delete ${pendingDelete?.fileName ?? 'this file'}`"
			body="The file is removed from this version of the shared instance. Members who install it afterwards won't receive it."
			stay-label="Cancel"
			leave-label="Delete file"
			admonition-type="critical"
		/>
	</NewModal>
</template>
