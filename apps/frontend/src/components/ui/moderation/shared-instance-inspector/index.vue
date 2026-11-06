<script setup lang="ts">
import {
	BanIcon,
	LinkIcon,
	LoaderCircleIcon,
	LockIcon,
	LockOpenIcon,
	SearchIcon,
	SortAscIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	type ButtonMenuOption,
	Combobox,
	type ComboboxOption,
	ConfirmLeaveModal,
	ContentCardTable,
	CopyCode,
	FilterPills,
	IconButton,
	injectModrinthClient,
	Input,
	NewModal,
	SplitButton,
	Tabs,
	type TabsTab,
	type TabsValue,
	useScrollIndicator,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

import {
	type SharedInstanceModerationUser,
	sharedInstanceQueryOptions,
} from '~/composables/queries/shared-instance'
import { useCopyText, useSharedInstanceModeration } from '~/composables/shared-instance-moderation'
import {
	formatSharedInstanceLoader,
	getRequestErrorMessage,
	sharedInstanceInspectorUrl,
} from '~/helpers/shared-instance-moderation'

import SharedInstanceBanModal from '../SharedInstanceBanModal.vue'
import SharedInstanceQuarantineModal from '../SharedInstanceQuarantineModal.vue'
import SharedInstanceUserInstancesPanel from '../SharedInstanceUserInstancesPanel.vue'
import SharedInstanceFilesPanel from './files-panel.vue'
import SharedInstanceMembersPanel from './members-panel.vue'
import SharedInstanceOwnerPanel from './owner-panel.vue'
import { type InspectorRow, sortOptions, useInspectorContent } from './use-inspector-content'

type InspectOptions = { version?: number; reportedVersions?: number[] }

const emit = defineEmits<{
	hide: []
	instanceChange: [instanceId: string]
}>()

const client = injectModrinthClient()
const queryClient = useQueryClient()
const copyText = useCopyText()
const {
	pending: actionPending,
	liftQuarantine,
	banUser,
	deleteFile,
} = useSharedInstanceModeration()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const quarantineModal = ref<InstanceType<typeof SharedInstanceQuarantineModal> | null>(null)
const deleteModal = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
const banModal = ref<InstanceType<typeof SharedInstanceBanModal> | null>(null)
const instanceId = ref<string | null>(null)
const selectedVersion = ref<number | null>(null)
const reportedVersions = ref<number[]>([])
const pendingDelete = ref<InspectorRow | null>(null)
const activeTab = ref<TabsValue>('content')
const openedTabs = ref(new Set<TabsValue>(['content']))
const tabs: TabsTab[] = [
	{ value: 'content', label: 'Content' },
	{ value: 'files', label: 'Files' },
	{ value: 'members', label: 'Members' },
	{ value: 'instances', label: 'Other instances' },
	{ value: 'owner', label: 'Owner info' },
]
const isVersionTab = computed(() => activeTab.value === 'content' || activeTab.value === 'files')
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

const { data: ownerBlacklist } = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.blacklist(details.value?.owner.id ?? '', client),
		enabled: details.value !== undefined,
	})),
)

const quarantineOptions: ButtonMenuOption[] = [
	{
		id: 'ban-owner',
		label: 'Ban from shared instances',
		icon: BanIcon,
		tone: 'red',
		action: promptBanOwner,
	},
]

const activeVersion = computed(() => selectedVersion.value ?? details.value?.versions[0]?.version)
const versionQueryEnabled = computed(
	() => instanceId.value !== null && activeVersion.value !== undefined,
)

const {
	data: rawVersion,
	error: versionError,
	isPending: versionPending,
	refetch: refetchVersion,
} = useQuery(
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

const {
	searchQuery,
	sortMode,
	selectedFilters,
	rows,
	visibleRows,
	tableItems,
	filterOptions,
	overrideFiles,
	resetContentFilters,
} = useInspectorContent(rawVersion, content)

watch(activeVersion, resetContentFilters)

watch(activeTab, (tab) => {
	openedTabs.value.add(tab)
})

const instanceSummary = computed(() => ({
	id: instanceId.value ?? '',
	name: details.value?.name ?? 'Shared instance',
	icon_url: details.value?.icon_url,
}))
const isInstanceQuarantined = computed(
	() => details.value?.quarantine_details?.scope === 'instance',
)

const versionOptions = computed<ComboboxOption<number>[]>(() =>
	(details.value?.versions ?? []).map((summary) => {
		const version = summary.version === rawVersion.value?.version ? rawVersion.value : summary
		return {
			value: version.version,
			label: `Version ${version.version}${reportedVersions.value.includes(version.version) ? ' · Reported' : ''}`,
			subLabel: version.game_version
				? `Minecraft ${version.game_version} · ${formatSharedInstanceLoader(version)}`
				: undefined,
		}
	}),
)

const memberRows = computed<SharedInstanceModerationUser[]>(() =>
	details.value ? [details.value.owner, ...details.value.members] : [],
)

async function handleDelete(id: string, event: MouseEvent) {
	const row = visibleRows.value.find((candidate) => candidate.tableItem.id === id)
	if (!row || row.kind === 'modrinth' || activeVersion.value === undefined) return

	pendingDelete.value = row
	if (!event.shiftKey && !(await deleteModal.value?.prompt())) return
	await deleteFile(instanceSummary.value, activeVersion.value, row.fileName, row.fileType)
}

async function promptBanOwner() {
	if (!details.value || !(await banModal.value?.prompt())) return
	await banUser(details.value.owner)
}

function copyInspectorLink() {
	if (!details.value) return
	void copyText(
		sharedInstanceInspectorUrl(details.value.owner.id, details.value.id, activeVersion.value),
		'Inspector link copied',
	)
}

function loadInstance(id: string, options: InspectOptions = {}) {
	instanceId.value = id
	selectedVersion.value = options.version ?? null
	reportedVersions.value = options.reportedVersions ?? []
	resetContentFilters()
	sortMode.value = 'uploaded'
	activeTab.value = 'content'
	openedTabs.value = new Set<TabsValue>(['content'])
}

function show(id: string, options: InspectOptions = {}) {
	loadInstance(id, options)
	modal.value?.show()
}

function inspectOtherInstance(id: string, versions: number[]) {
	loadInstance(id, { reportedVersions: versions })
	emit('instanceChange', id)
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
				<span class="truncate text-2xl font-semibold text-contrast">{{
					instanceSummary.name
				}}</span>
				<CopyCode v-if="instanceId" v-tooltip="'Copy instance ID'" :text="instanceId" />
			</div>
		</template>
		<template #header-actions>
			<div v-if="details" class="flex items-center gap-2">
				<div v-if="versionOptions.length && isVersionTab" class="w-56 shrink-0">
					<Combobox
						:model-value="activeVersion"
						:options="versionOptions"
						dropdown-min-width="18rem"
						dropdown-align="right"
						@update:model-value="(value) => (selectedVersion = value as number)"
					/>
				</div>
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
				<SplitButton
					v-else
					type="colored"
					color="orange"
					menu-label="More moderation actions"
					:options="quarantineOptions"
					:disabled="actionPending"
					:menu-disabled="ownerBlacklist?.blacklisted !== false"
					@click="quarantineModal?.show()"
				>
					<LockIcon />
					Quarantine
				</SplitButton>
				<IconButton
					v-tooltip="'Copy inspector link'"
					label="Copy inspector link"
					@click="copyInspectorLink"
				>
					<LinkIcon />
				</IconButton>
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
		<div v-else-if="details" class="flex h-[min(720px,calc(95vh-8rem))] flex-col overflow-hidden">
			<section aria-label="Shared instance details" class="flex min-h-0 min-w-0 flex-1 flex-col">
				<div
					class="flex shrink-0 flex-col gap-4 border-0 border-b border-solid border-surface-4 px-6 py-4"
				>
					<Tabs v-model:value="activeTab" :tabs="tabs" size="large" />
				</div>
				<div
					v-if="activeTab === 'content'"
					class="flex shrink-0 flex-col gap-4 border-0 border-b border-solid border-surface-4 px-6 py-4"
				>
					<Input
						v-model="searchQuery"
						:icon="SearchIcon"
						type="search"
						:placeholder="`Search ${rows.length} ${rows.length === 1 ? 'item' : 'items'}`"
						:aria-label="`Search ${rows.length} ${rows.length === 1 ? 'item' : 'items'}`"
						clearable
					/>
					<div class="flex flex-wrap items-center gap-2">
						<div class="w-44 shrink-0">
							<Combobox
								v-model="sortMode"
								:options="sortOptions"
								trigger-size="sm"
								aria-label="Sort content"
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

				<div v-if="!versionQueryEnabled && isVersionTab" class="p-8 text-center text-secondary">
					No versions have been published.
				</div>
				<template v-if="versionQueryEnabled">
					<template v-if="activeTab === 'content'">
						<div
							v-if="contentPending"
							class="flex flex-1 items-center justify-center gap-2 p-8 text-secondary"
						>
							<LoaderCircleIcon class="size-6 animate-spin" />
							Loading content…
						</div>
						<div v-else-if="contentError" class="p-6">
							<Admonition
								type="critical"
								header="Failed to load content"
								:body="
									getRequestErrorMessage(
										contentError,
										'The content for this version could not be loaded.',
									)
								"
							/>
						</div>
						<div v-else-if="!rows.length" class="p-8 text-center text-secondary">
							This version has no content.
						</div>
						<div v-else-if="!visibleRows.length" class="p-8 text-center text-secondary">
							No content matches your search.
						</div>
						<div v-else class="relative min-h-0 flex-1">
							<div ref="tableScroll" class="h-full overflow-y-auto px-6 pb-6 @container">
								<ContentCardTable class="mt-4" :items="tableItems" @delete="handleDelete" />
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
					</template>
					<SharedInstanceFilesPanel
						v-if="openedTabs.has('files')"
						v-show="activeTab === 'files'"
						:key="`${instanceSummary.id}:${activeVersion}`"
						class="min-h-0 flex-1"
						:instance="instanceSummary"
						:version="activeVersion ?? 0"
						:files="overrideFiles"
						:loading="versionPending"
						:error="versionError"
						@refresh="refetchVersion()"
					/>
				</template>
				<SharedInstanceMembersPanel
					v-if="openedTabs.has('members')"
					v-show="activeTab === 'members'"
					class="min-h-0 flex-1"
					:members="memberRows"
				/>
				<SharedInstanceUserInstancesPanel
					v-if="openedTabs.has('instances')"
					v-show="activeTab === 'instances'"
					class="min-h-0 flex-1 overflow-y-auto p-6"
					:user-id="details.owner.id"
					:exclude-instance-id="details.id"
					@inspect="inspectOtherInstance"
				/>
				<SharedInstanceOwnerPanel
					v-if="openedTabs.has('owner')"
					v-show="activeTab === 'owner'"
					class="min-h-0 flex-1"
					:owner="details.owner"
				/>
			</section>
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
		<SharedInstanceBanModal
			ref="banModal"
			:username="details?.owner.username ?? 'owner'"
			subject="owner"
		/>
	</NewModal>
</template>
