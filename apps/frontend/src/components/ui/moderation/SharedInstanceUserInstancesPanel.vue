<template>
	<div class="flex flex-col gap-4">
		<SharedInstanceQuarantineModal
			v-if="quarantineTarget"
			ref="quarantineModal"
			:instance="quarantineTarget"
		/>

		<div
			v-if="isPending"
			class="flex min-h-48 items-center justify-center gap-2 rounded-2xl border border-solid border-surface-4 bg-surface-3 text-secondary"
		>
			<LoaderCircleIcon class="size-6 animate-spin" />
			Loading shared instances…
		</div>

		<Admonition
			v-else-if="error"
			type="critical"
			header="Failed to load shared instances"
			:body="getRequestErrorMessage(error, 'The shared instances service could not be reached.')"
			show-actions-underneath
		>
			<template #actions>
				<Button :disabled="isFetching" @click="refetch()">
					<LoaderCircleIcon v-if="isFetching" class="animate-spin" />
					Try again
				</Button>
			</template>
		</Admonition>

		<template v-else>
			<Admonition
				v-if="sharedInstances?.unavailableCount"
				type="warning"
				header="Some shared instances are unavailable"
				:body="`${sharedInstances.unavailableCount} ${sharedInstances.unavailableCount === 1 ? 'instance could' : 'instances could'} not be loaded.`"
			/>

			<div class="flex flex-wrap items-center justify-between gap-3">
				<FilterPills
					:model-value="selectedFilters"
					:options="filterOptions"
					@update:model-value="(filters) => (selectedFilters = filters.slice(-1))"
				>
					<template #all>All {{ rows.length }}</template>
				</FilterPills>
				<span
					v-if="blacklist"
					class="text-sm"
					:class="blacklist.blacklisted ? 'font-semibold text-red' : 'text-secondary'"
				>
					{{ banStatusText }}
					<template v-if="blacklist.blacklisted && blacklist.blacklisted_at">
						· {{ formatDateTime(blacklist.blacklisted_at) }}
					</template>
				</span>
			</div>

			<div
				v-if="!rows.length"
				class="flex min-h-48 flex-col items-center justify-center gap-2 rounded-2xl border border-solid border-surface-4 bg-surface-3 p-6 text-center"
			>
				<BoxesIcon class="size-10 text-secondary" />
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ excludeInstanceId ? 'No other shared instances' : 'No shared instances' }}
				</h2>
				<p class="m-0 text-secondary">
					This user does not own or belong to any
					{{ excludeInstanceId ? 'other ' : '' }}shared instances.
				</p>
			</div>

			<Table v-else :columns="columns" :data="visibleRows" row-key="id" table-min-width="65rem">
				<template #cell-instance="{ row }">
					<div class="flex min-w-0 items-center gap-3 py-2.5">
						<Avatar
							:src="row.iconUrl"
							:alt="row.name"
							:tint-by="row.id"
							size="40px"
							no-shadow
							pad-transparent-corners
						/>
						<div class="flex min-w-0 flex-col items-start gap-1">
							<button
								type="button"
								class="max-w-full cursor-pointer truncate border-0 bg-transparent p-0 text-left font-semibold text-contrast hover:underline"
								@click="inspect(row)"
							>
								{{ row.name }}
							</button>
							<CopyCode :text="row.id" />
						</div>
					</div>
				</template>
				<template #cell-role="{ row }">
					{{ roleLabel(row) }}
				</template>
				<template #cell-version="{ row }">
					<span
						v-if="row.latestVersion"
						v-tooltip="
							`Minecraft ${row.latestVersion.game_version} · ${formatSharedInstanceLoader(row.latestVersion)}`
						"
					>
						Version {{ row.latestVersion.version }}
					</span>
					<span v-else>Unavailable</span>
				</template>
				<template #cell-members="{ row }">
					{{ row.memberCount }}
				</template>
				<template #cell-lastPlayed="{ row }">
					<span
						v-if="row.membership?.last_played"
						v-tooltip="formatDateTime(row.membership.last_played)"
					>
						{{ formatRelativeTime(row.membership.last_played) }}
					</span>
					<span v-else>Never</span>
				</template>
				<template #cell-status="{ row }">
					<div class="flex flex-wrap items-center gap-1.5">
						<span
							v-if="row.quarantine"
							class="bg-orange-highlight inline-flex items-center gap-1 whitespace-nowrap rounded-full px-2 py-0.5 text-xs font-semibold text-orange"
						>
							<LockIcon class="size-3.5" />
							Quarantined
						</span>
						<NuxtLink
							v-if="row.openReports.length"
							:to="`/moderation/reports/${row.openReports[0].id}`"
							target="_blank"
							class="border-blue/60 inline-flex items-center whitespace-nowrap rounded-full border border-solid bg-highlight-blue px-2 py-0.5 text-xs font-semibold text-blue hover:underline"
						>
							{{
								row.openReports.length === 1
									? 'Open report'
									: `${row.openReports.length} open reports`
							}}
						</NuxtLink>
						<span v-if="!row.quarantine && !row.openReports.length">Active</span>
					</div>
				</template>
				<template #cell-actions="{ row }">
					<div class="flex justify-end gap-1">
						<Button @click="inspect(row)">
							<EyeIcon />
							Inspect
						</Button>
						<TeleportOverflowMenu
							type="quiet"
							:label="`More options for ${row.name}`"
							:options="rowOptions(row)"
						>
							<MoreVerticalIcon />
						</TeleportOverflowMenu>
					</div>
				</template>
			</Table>
		</template>
	</div>
</template>

<script setup lang="ts">
import {
	BoxesIcon,
	EyeIcon,
	LinkIcon,
	LoaderCircleIcon,
	LockIcon,
	LockOpenIcon,
	MoreVerticalIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	type ButtonMenuOption,
	CopyCode,
	FilterPills,
	injectModrinthClient,
	Table,
	type TableColumn,
	TeleportOverflowMenu,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, nextTick, onServerPrefetch, ref } from 'vue'

import {
	sharedInstanceQueryOptions,
	type UserSharedInstance,
} from '~/composables/queries/shared-instance'
import { useCopyText, useSharedInstanceModeration } from '~/composables/shared-instance-moderation'
import {
	formatSharedInstanceLoader,
	getRequestErrorMessage,
	sharedInstanceInspectorUrl,
} from '~/helpers/shared-instance-moderation'

import SharedInstanceQuarantineModal from './SharedInstanceQuarantineModal.vue'

type ColumnKey = 'instance' | 'role' | 'version' | 'members' | 'lastPlayed' | 'status' | 'actions'
type InstanceRow = UserSharedInstance & { openReports: { id: string; version?: number }[] }
type FilterId = 'reported' | 'quarantined' | 'member'

const props = defineProps<{
	userId: string
	excludeInstanceId?: string
}>()

const emit = defineEmits<{
	inspect: [instanceId: string, reportedVersions: number[]]
}>()

const client = injectModrinthClient()
const copyText = useCopyText()
const { liftQuarantine } = useSharedInstanceModeration()
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const canLoadUser = computed(() => props.userId.length > 0)
const quarantineModal = ref<InstanceType<typeof SharedInstanceQuarantineModal> | null>(null)
const quarantineTarget = ref<{ id: string; name: string; icon_url: string | null } | null>(null)
const selectedFilters = ref<string[]>([])

const columns: TableColumn<ColumnKey>[] = [
	{ key: 'instance', label: 'Instance' },
	{ key: 'role', label: 'Role', width: '6rem' },
	{ key: 'version', label: 'Version', width: '7rem' },
	{ key: 'members', label: 'Members', width: '6rem' },
	{ key: 'lastPlayed', label: 'Last played', width: '8rem' },
	{ key: 'status', label: 'Status', width: '10rem' },
	{ key: 'actions', label: 'Actions', width: '10rem', align: 'right' },
]

const {
	data: sharedInstances,
	error,
	isPending,
	isFetching,
	refetch,
	suspense,
} = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.userInstances(props.userId, client),
		enabled: canLoadUser.value,
	})),
)

const { data: blacklist } = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.blacklist(props.userId, client),
		enabled: canLoadUser.value,
	})),
)

const listedInstances = computed(() =>
	(sharedInstances.value?.instances ?? []).filter(
		(instance) => instance.id !== props.excludeInstanceId,
	),
)
const instanceIds = computed(() => listedInstances.value.map((instance) => instance.id))
const { data: openReports } = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.openReports(instanceIds.value, client),
		enabled: import.meta.client && instanceIds.value.length > 0,
	})),
)

const bannedById = computed(() => blacklist.value?.blacklisted_by ?? null)
const { data: bannedBy } = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.moderator(bannedById.value ?? '', client),
		enabled: bannedById.value !== null,
	})),
)
const banStatusText = computed(() => {
	if (!blacklist.value?.blacklisted) return 'Account allowed to use shared instances'
	return bannedBy.value
		? `Banned from shared instances by ${bannedBy.value.username}`
		: 'Banned from shared instances'
})

const rows = computed<InstanceRow[]>(() => {
	const reportsByInstance = new Map<string, InstanceRow['openReports']>()
	for (const report of openReports.value ?? []) {
		const reports = reportsByInstance.get(report.item_id) ?? []
		reports.push({ id: report.id, version: report.shared_instance_version_id })
		reportsByInstance.set(report.item_id, reports)
	}
	return listedInstances.value.map((instance) => ({
		...instance,
		openReports: reportsByInstance.get(instance.id) ?? [],
	}))
})

const filterMatchers: Record<FilterId, (row: InstanceRow) => boolean> = {
	reported: (row) => row.openReports.length > 0,
	quarantined: (row) => row.quarantine,
	member: (row) => row.membership?.join_type !== 'owner',
}

const filterOptions = computed(() =>
	(
		[
			['reported', 'Reported'],
			['quarantined', 'Quarantined'],
			['member', 'Member of'],
		] as const
	).map(([id, label]) => ({
		id,
		label: `${label} ${rows.value.filter(filterMatchers[id]).length}`,
	})),
)

const visibleRows = computed(() => {
	const filter = selectedFilters.value[0] as FilterId | undefined
	return filter ? rows.value.filter(filterMatchers[filter]) : rows.value
})

onServerPrefetch(suspense)

function roleLabel(row: InstanceRow) {
	if (!row.membership) return 'Unknown'
	return row.membership.join_type === 'owner' ? 'Owner' : 'Member'
}

function inspect(row: InstanceRow) {
	emit(
		'inspect',
		row.id,
		row.openReports.flatMap((report) => (report.version !== undefined ? [report.version] : [])),
	)
}

async function openQuarantine(row: InstanceRow) {
	quarantineTarget.value = { id: row.id, name: row.name, icon_url: row.iconUrl }
	await nextTick()
	quarantineModal.value?.show()
}

function rowOptions(row: InstanceRow): ButtonMenuOption[] {
	return [
		{
			id: 'lift-quarantine',
			label: 'Lift quarantine',
			icon: LockOpenIcon,
			shown: row.quarantineScope === 'instance',
			action: () => liftQuarantine(row),
		},
		{
			id: 'quarantine',
			label: 'Quarantine',
			icon: LockIcon,
			tone: 'orange',
			shown: row.quarantineScope === null,
			action: () => openQuarantine(row),
		},
		{
			id: 'copy-link',
			label: 'Copy inspector link',
			icon: LinkIcon,
			action: () =>
				copyText(sharedInstanceInspectorUrl(props.userId, row.id), 'Inspector link copied'),
		},
	]
}
</script>
