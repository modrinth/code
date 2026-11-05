<template>
	<SharedInstanceInspectorModal ref="inspectorModal" @hide="clearInspectorQuery" />
	<SharedInstanceQuarantineModal
		v-if="quarantineTarget"
		ref="quarantineModal"
		:instance="quarantineTarget"
	/>
	<ConfirmLeaveModal
		ref="banModal"
		title="Ban from shared instances?"
		:header="`Ban ${username} from shared instances`"
		body="This prevents the user from creating or managing shared instances. All of their shared instances will be quarantined, and members won't be able to launch them."
		stay-label="Cancel"
		leave-label="Ban user"
		admonition-type="critical"
	/>

	<div class="normal-page no-sidebar !pb-6">
		<div class="normal-page__content flex flex-col gap-4">
			<header
				class="flex flex-col gap-3 border-0 border-b border-solid border-divider pb-4 sm:flex-row sm:items-center sm:justify-between"
			>
				<div class="flex min-w-0 items-center gap-3">
					<Avatar
						:src="user?.avatar_url"
						:alt="username"
						:tint-by="user?.id ?? userId"
						size="48px"
						circle
					/>
					<h1 class="m-0 min-w-0 truncate text-2xl font-extrabold text-contrast">
						{{ username }}'s shared instances
					</h1>
				</div>

				<div class="flex flex-wrap gap-2">
					<ButtonLink :to="`/user/${user?.id ?? userId}`" target="_blank">
						<UserIcon />
						User profile
						<ExternalIcon class="size-4" />
					</ButtonLink>
					<Button
						v-if="blacklist?.blacklisted"
						type="outlined"
						:loading="actionPending"
						@click="unbanUser(userSummary)"
					>
						<UndoIcon />
						Unban from shared instances
					</Button>
					<Button
						v-else
						type="outlined"
						color="red"
						:disabled="!blacklist"
						:loading="actionPending"
						@click="promptBan"
					>
						<BanIcon />
						Ban from shared instances
					</Button>
				</div>
			</header>

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
				:body="errorMessage"
				show-actions-underneath
			>
				<template #actions>
					<Button :disabled="isFetching" @click="retry">
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
					<h2 class="m-0 text-lg font-semibold text-contrast">No shared instances</h2>
					<p class="m-0 text-secondary">
						This user does not own or belong to any shared instances.
					</p>
				</div>

				<Table
					v-else
					:columns="columns"
					:data="visibleRows"
					row-key="id"
					table-min-width="65rem"
				>
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
									@click="openInspector(row)"
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
							<Button @click="openInspector(row)">
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
	</div>
</template>

<script setup lang="ts">
import {
	BanIcon,
	BoxesIcon,
	ExternalIcon,
	EyeIcon,
	LinkIcon,
	LoaderCircleIcon,
	LockIcon,
	LockOpenIcon,
	MoreVerticalIcon,
	UndoIcon,
	UserIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	ButtonLink,
	type ButtonMenuOption,
	ConfirmLeaveModal,
	CopyCode,
	FilterPills,
	injectModrinthClient,
	injectNotificationManager,
	Table,
	type TableColumn,
	TeleportOverflowMenu,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, nextTick, onMounted, onServerPrefetch, ref } from 'vue'

import SharedInstanceInspectorModal from '~/components/ui/moderation/SharedInstanceInspectorModal.vue'
import SharedInstanceQuarantineModal from '~/components/ui/moderation/SharedInstanceQuarantineModal.vue'
import {
	sharedInstanceQueryOptions,
	type UserSharedInstance,
} from '~/composables/queries/shared-instance'
import {
	formatSharedInstanceLoader,
	getRequestErrorMessage,
	sharedInstanceInspectorPath,
	useSharedInstanceModeration,
} from '~/composables/shared-instance-moderation'

type ColumnKey = 'instance' | 'role' | 'version' | 'members' | 'lastPlayed' | 'status' | 'actions'
type InstanceRow = UserSharedInstance & { openReports: { id: string; version?: number }[] }
type FilterId = 'reported' | 'quarantined' | 'member'

const route = useRoute()
const router = useRouter()
const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const {
	pending: actionPending,
	liftQuarantine,
	banUser,
	unbanUser,
} = useSharedInstanceModeration()
const formatRelativeTime = useRelativeTime({ style: 'narrow' })
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

const userId = computed(() => String(route.params.user ?? ''))
const canLoadUser = computed(() => userId.value.length > 0)
const inspectorModal = ref<InstanceType<typeof SharedInstanceInspectorModal> | null>(null)
const quarantineModal = ref<InstanceType<typeof SharedInstanceQuarantineModal> | null>(null)
const banModal = ref<InstanceType<typeof ConfirmLeaveModal> | null>(null)
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
	data: user,
	error: userError,
	isPending: userPending,
	isFetching: userFetching,
	refetch: refetchUser,
	suspense: userSuspense,
} = useQuery({
	queryKey: computed(() => ['user', 'v3', userId.value] as const),
	queryFn: () => client.labrinth.users_v3.get(userId.value),
	enabled: canLoadUser,
})

const {
	data: sharedInstances,
	error: sharedInstancesError,
	isPending: sharedInstancesPending,
	isFetching: sharedInstancesFetching,
	refetch: refetchSharedInstances,
	suspense: sharedInstancesSuspense,
} = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.userInstances(userId.value, client),
		enabled: canLoadUser.value,
	})),
)

const { data: blacklist } = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.blacklist(userId.value, client),
		enabled: canLoadUser.value,
	})),
)

const instanceIds = computed(() =>
	(sharedInstances.value?.instances ?? []).map((instance) => instance.id),
)
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

const username = computed(() => user.value?.username ?? userId.value)
const userSummary = computed(() => ({ id: user.value?.id ?? userId.value, username: username.value }))

const rows = computed<InstanceRow[]>(() => {
	const reportsByInstance = new Map<string, InstanceRow['openReports']>()
	for (const report of openReports.value ?? []) {
		const reports = reportsByInstance.get(report.item_id) ?? []
		reports.push({ id: report.id, version: report.shared_instance_version_id })
		reportsByInstance.set(report.item_id, reports)
	}
	return (sharedInstances.value?.instances ?? []).map((instance) => ({
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

const isPending = computed(() => userPending.value || sharedInstancesPending.value)
const isFetching = computed(() => userFetching.value || sharedInstancesFetching.value)
const error = computed(() => userError.value ?? sharedInstancesError.value)
const errorMessage = computed(() =>
	getRequestErrorMessage(error.value, 'The shared instances service could not be reached.'),
)

useHead({
	title: computed(() => `${username.value}'s shared instances - Modrinth`),
})

onServerPrefetch(() => Promise.all([userSuspense(), sharedInstancesSuspense()]))

onMounted(() => {
	const instanceId = route.query.instance
	if (typeof instanceId !== 'string' || !instanceId) return
	const version = Number(route.query.version)
	inspectorModal.value?.show(instanceId, {
		version: Number.isInteger(version) ? version : undefined,
	})
})

function roleLabel(row: InstanceRow) {
	if (!row.membership) return 'Unknown'
	return row.membership.join_type === 'owner' ? 'Owner' : 'Member'
}

function openInspector(row: InstanceRow) {
	inspectorModal.value?.show(row.id, {
		reportedVersions: row.openReports.flatMap((report) =>
			report.version !== undefined ? [report.version] : [],
		),
	})
	void router.replace({ query: { ...route.query, instance: row.id, version: undefined } })
}

function clearInspectorQuery() {
	if (route.query.instance === undefined) return
	void router.replace({ query: { ...route.query, instance: undefined, version: undefined } })
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
			action: () => copyInspectorLink(row),
		},
	]
}

async function copyInspectorLink(row: InstanceRow) {
	const url = `${window.location.origin}${sharedInstanceInspectorPath(userId.value, row.id)}`
	await navigator.clipboard.writeText(url)
	addNotification({ type: 'success', title: 'Inspector link copied', text: url })
}

async function promptBan() {
	if (!(await banModal.value?.prompt())) return
	await banUser(userSummary.value)
}

function retry() {
	void Promise.all([refetchUser(), refetchSharedInstances()])
}
</script>
