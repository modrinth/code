<template>
	<div class="flex flex-col gap-4">
		<div class="flex items-center">
			<Combobox
				v-model="selectedTimeRange"
				class="!w-full sm:!w-[245px]"
				:options="timeRangeOptions"
				:display-value="selectedTimeRangeLabel"
				trigger-class="!h-10 !w-full sm:!w-[245px] !rounded-[14px] !bg-surface-4 !px-4 !py-2.5 !text-base shadow-[0px_1px_1px_rgba(0,0,0,0.3),0px_1px_1.5px_rgba(0,0,0,0.15)]"
				dropdown-min-width="245px"
			/>
		</div>

		<Table
			v-if="filteredEntries.length > 0"
			class="hidden sm:block"
			:columns="columns"
			:data="tableEntries"
			row-key="id"
		>
			<template #cell-user="{ row: entry }">
				<AutoLink
					:to="actorProfilePath(entry)"
					class="flex min-w-0 items-center gap-2"
					:class="actorProfilePath(entry) ? 'text-primary hover:underline' : ''"
				>
					<Avatar
						:src="
							entry.actor.id === 'support'
								? IntercomBubbleIcon
								: (entry.actor.avatarUrl ?? undefined)
						"
						:alt="formatMessage(messages.userAvatarAlt, { username: entry.actor.username })"
						:tint-by="entry.actor.username"
						size="22px"
						circle
						no-shadow
					/>
					<span
						class="min-w-0 truncate font-medium"
						:class="entry.actor.id === 'support' ? 'text-blue' : ''"
					>
						{{
							entry.actor.id === 'support'
								? formatMessage(messages.supportActor)
								: entry.actor.username
						}}
					</span>
				</AutoLink>
			</template>

			<template #cell-world="{ row: entry }">
				<span
					v-tooltip="entry.world?.name"
					class="truncate"
					:class="entry.world ? 'text-primary' : 'text-secondary'"
				>
					{{ entry.world?.name ?? '—' }}
				</span>
			</template>

			<template #cell-action="{ row: entry }">
				<div v-tooltip="actionTooltip(entry, 'desktop')" class="flex min-w-0 items-center gap-1.5">
					<template v-if="entry.action.type === 'content_installed'">
						<span class="shrink-0 text-secondary">{{ contentInstalledPrefix(entry.action) }}</span>
						<AutoLink
							:to="entry.action.href"
							class="inline-flex min-w-0 items-center gap-1.5"
							:class="actionLinkClass(entry.action.href)"
						>
							<Avatar
								:src="entry.action.iconUrl ?? undefined"
								:alt="formatMessage(messages.contentIconAlt, { name: entry.action.name })"
								:tint-by="entry.action.name"
								size="1.5rem"
								no-shadow
							/>
							<span
								:ref="
									(element) => setActionLabelRef(actionLabelRefKey(entry.id, 'desktop'), element)
								"
								class="min-w-0 truncate font-medium"
							>
								{{ entry.action.name }}
							</span>
						</AutoLink>
						<span v-if="entry.action.version" class="shrink-0 text-secondary">
							{{ formatVersionSuffix(entry.action.version) }}
						</span>
					</template>
					<template v-else-if="isMemberAuditAction(entry.action)">
						<span class="shrink-0 text-secondary">{{ memberActionPrefix(entry.action) }}</span>
						<AutoLink
							:to="userProfilePath(entry.action.target)"
							class="inline-flex min-w-0 items-center gap-1.5"
							:class="actionLinkClass(userProfilePath(entry.action.target))"
						>
							<Avatar
								:src="targetAvatarSrc(entry.action.target)"
								:alt="formatMessage(messages.userAvatarAlt, { username: entry.action.target })"
								:tint-by="entry.action.target"
								size="1.5rem"
								circle
								no-shadow
							/>
							<span
								:ref="
									(element) => setActionLabelRef(actionLabelRefKey(entry.id, 'desktop'), element)
								"
								class="min-w-0 truncate font-medium"
							>
								{{ entry.action.target }}
							</span>
						</AutoLink>
						<span v-if="memberActionSuffix(entry.action)" class="shrink-0 text-secondary">
							{{ memberActionSuffix(entry.action) }}
						</span>
					</template>
					<span
						v-else
						:ref="(element) => setActionLabelRef(actionLabelRefKey(entry.id, 'desktop'), element)"
						class="min-w-0 truncate"
					>
						{{ formatAction(entry.action) }}
					</span>
				</div>
			</template>

			<template #cell-time="{ row: entry }">
				<span v-tooltip="formatDate(entry.timestamp)">
					{{ formatRelativeTime(entry.timestamp) }}
				</span>
			</template>
		</Table>

		<div
			v-if="filteredEntries.length > 0"
			class="overflow-hidden rounded-2xl border border-solid border-surface-5 sm:hidden"
		>
			<div class="grid min-h-14 grid-cols-[3.75rem_minmax(0,1fr)_5rem] bg-surface-3">
				<div class="flex items-center pl-4 font-semibold text-secondary">
					{{ formatMessage(messages.userColumn) }}
				</div>
				<div class="flex items-center font-semibold text-secondary">
					{{ formatMessage(messages.actionColumn) }}
				</div>
				<div class="flex items-center justify-end pr-4 font-semibold text-secondary">
					{{ formatMessage(messages.timeColumn) }}
				</div>
			</div>
			<div
				v-for="(entry, index) in filteredEntries"
				:key="entry.id"
				class="grid min-h-[4.5rem] grid-cols-[3.75rem_minmax(0,1fr)_5rem] items-center border-0 border-t border-solid border-surface-5"
				:class="index % 2 === 0 ? 'bg-surface-2' : 'bg-surface-1.5'"
			>
				<div class="flex min-w-0 items-center pl-4">
					<AutoLink
						v-tooltip="actorName(entry)"
						:to="actorProfilePath(entry)"
						class="inline-flex shrink-0"
					>
						<Avatar
							:src="actorAvatarSrc(entry)"
							:alt="formatMessage(messages.userAvatarAlt, { username: actorName(entry) })"
							:tint-by="entry.actor.username"
							size="24px"
							circle
							no-shadow
						/>
					</AutoLink>
				</div>
				<div class="min-w-0 py-3 pr-2">
					<div v-tooltip="actionTooltip(entry, 'mobile')" class="flex min-w-0 items-center gap-1.5">
						<template v-if="entry.action.type === 'content_installed'">
							<span class="shrink-0 text-secondary">{{
								contentInstalledPrefix(entry.action)
							}}</span>
							<AutoLink
								:to="entry.action.href"
								class="inline-flex min-w-0 items-center gap-1.5"
								:class="actionLinkClass(entry.action.href)"
							>
								<Avatar
									:src="entry.action.iconUrl ?? undefined"
									:alt="formatMessage(messages.contentIconAlt, { name: entry.action.name })"
									:tint-by="entry.action.name"
									size="1.25rem"
									no-shadow
								/>
								<span
									:ref="
										(element) => setActionLabelRef(actionLabelRefKey(entry.id, 'mobile'), element)
									"
									class="min-w-0 truncate font-medium"
								>
									{{ entry.action.name }}
								</span>
							</AutoLink>
							<span v-if="entry.action.version" class="shrink-0 text-secondary">
								{{ formatVersionSuffix(entry.action.version) }}
							</span>
						</template>
						<template v-else-if="isMemberAuditAction(entry.action)">
							<span class="shrink-0 text-secondary">{{ memberActionPrefix(entry.action) }}</span>
							<AutoLink
								:to="userProfilePath(entry.action.target)"
								class="inline-flex min-w-0 items-center gap-1.5"
								:class="actionLinkClass(userProfilePath(entry.action.target))"
							>
								<Avatar
									:src="targetAvatarSrc(entry.action.target)"
									:alt="formatMessage(messages.userAvatarAlt, { username: entry.action.target })"
									:tint-by="entry.action.target"
									size="1.25rem"
									circle
									no-shadow
								/>
								<span
									:ref="
										(element) => setActionLabelRef(actionLabelRefKey(entry.id, 'mobile'), element)
									"
									class="min-w-0 truncate font-medium"
								>
									{{ entry.action.target }}
								</span>
							</AutoLink>
							<span v-if="memberActionSuffix(entry.action)" class="shrink-0 text-secondary">
								{{ memberActionSuffix(entry.action) }}
							</span>
						</template>
						<span
							v-else
							:ref="(element) => setActionLabelRef(actionLabelRefKey(entry.id, 'mobile'), element)"
							class="min-w-0 truncate text-secondary"
						>
							{{ formatAction(entry.action) }}
						</span>
					</div>
					<span
						v-tooltip="entry.world?.name"
						class="mt-1 block truncate text-sm"
						:class="entry.world ? 'text-primary' : 'text-secondary'"
					>
						{{ entry.world?.name ?? '—' }}
					</span>
				</div>
				<div class="min-w-0 py-3 pr-4 text-right text-secondary">
					<span v-tooltip="formatDate(entry.timestamp)" class="inline-block max-w-full truncate">
						{{ formatRelativeTime(entry.timestamp) }}
					</span>
				</div>
			</div>
		</div>

		<div v-else class="overflow-hidden rounded-2xl border border-solid border-surface-5">
			<div
				class="grid min-h-14 grid-cols-[3.75rem_minmax(0,1fr)_5rem] bg-surface-3 sm:h-14 sm:grid-cols-[20%_18%_44%_18%]"
			>
				<div class="flex items-center pl-4 font-semibold text-secondary">
					{{ formatMessage(messages.userColumn) }}
				</div>
				<div class="hidden items-center font-semibold text-secondary sm:flex">
					{{ formatMessage(messages.worldColumn) }}
				</div>
				<div class="flex items-center font-semibold text-secondary">
					{{ formatMessage(messages.actionColumn) }}
				</div>
				<div class="flex items-center justify-end pr-4 font-semibold text-secondary">
					{{ formatMessage(messages.timeColumn) }}
				</div>
			</div>
			<div
				class="border-0 border-t border-solid border-surface-5 bg-surface-2 px-4 py-8 text-center text-secondary"
			>
				{{ formatMessage(messages.emptyState) }}
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { IntercomBubbleIcon } from '@modrinth/assets'
import { computed, ref, shallowReactive } from 'vue'

import { useFormatDateTime, useRelativeTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { truncatedTooltip } from '../../../utils/truncate'
import AutoLink from '../../base/AutoLink.vue'
import Avatar from '../../base/Avatar.vue'
import Combobox, { type ComboboxOption } from '../../base/Combobox.vue'
import Table, { type TableColumn } from '../../base/Table.vue'
import type {
	ServerAccessMember,
	ServerAccessRole,
	ServerAuditAction,
	ServerAuditLogEntry,
	ServerAuditLogFilters,
} from './types'

const props = defineProps<{
	entries: ServerAuditLogEntry[]
	users: ServerAccessMember[]
	worlds: { id: string; name: string }[]
}>()

const query = defineModel<string>('query', { default: '' })
const filters = defineModel<ServerAuditLogFilters>('filters', {
	default: () => ({
		userId: null,
		worldId: null,
		actionType: null,
	}),
})

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDate = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

type AuditTimeRange =
	| 'previous_30_minutes'
	| 'previous_hour'
	| 'previous_12_hours'
	| 'previous_24_hours'
	| 'today'
	| 'yesterday'
	| 'this_week'
	| 'last_week'
	| 'previous_7_days'
	| 'this_month'
	| 'last_month'
	| 'last_30_days'
	| 'this_quarter'
	| 'last_quarter'
	| 'this_year'
	| 'last_year'
	| 'previous_year'
	| 'previous_two_years'
	| 'all_time'

const selectedTimeRange = ref<AuditTimeRange>('last_30_days')

const messages = defineMessages({
	previous30Minutes: {
		id: 'servers.audit-log.time-range.previous-30-minutes',
		defaultMessage: 'Previous 30 minutes',
	},
	previousHour: {
		id: 'servers.audit-log.time-range.previous-hour',
		defaultMessage: 'Previous hour',
	},
	previous12Hours: {
		id: 'servers.audit-log.time-range.previous-12-hours',
		defaultMessage: 'Previous 12 hours',
	},
	previous24Hours: {
		id: 'servers.audit-log.time-range.previous-24-hours',
		defaultMessage: 'Previous 24 hours',
	},
	today: {
		id: 'servers.audit-log.time-range.today',
		defaultMessage: 'Today',
	},
	yesterday: {
		id: 'servers.audit-log.time-range.yesterday',
		defaultMessage: 'Yesterday',
	},
	thisWeek: {
		id: 'servers.audit-log.time-range.this-week',
		defaultMessage: 'This week',
	},
	lastWeek: {
		id: 'servers.audit-log.time-range.last-week',
		defaultMessage: 'Last week',
	},
	previous7Days: {
		id: 'servers.audit-log.time-range.previous-7-days',
		defaultMessage: 'Previous 7 days',
	},
	thisMonth: {
		id: 'servers.audit-log.time-range.this-month',
		defaultMessage: 'This month',
	},
	lastMonth: {
		id: 'servers.audit-log.time-range.last-month',
		defaultMessage: 'Last month',
	},
	last30Days: {
		id: 'servers.audit-log.time-range.last-30-days',
		defaultMessage: 'Last 30 days',
	},
	thisQuarter: {
		id: 'servers.audit-log.time-range.this-quarter',
		defaultMessage: 'This quarter',
	},
	lastQuarter: {
		id: 'servers.audit-log.time-range.last-quarter',
		defaultMessage: 'Last quarter',
	},
	thisYear: {
		id: 'servers.audit-log.time-range.this-year',
		defaultMessage: 'This year',
	},
	lastYear: {
		id: 'servers.audit-log.time-range.last-year',
		defaultMessage: 'Last year',
	},
	previousYear: {
		id: 'servers.audit-log.time-range.previous-year',
		defaultMessage: 'Previous year',
	},
	previousTwoYears: {
		id: 'servers.audit-log.time-range.previous-two-years',
		defaultMessage: 'Previous two years',
	},
	allTime: {
		id: 'servers.audit-log.time-range.all-time',
		defaultMessage: 'All Time',
	},
	supportActor: {
		id: 'servers.audit-log.actor.support',
		defaultMessage: 'Support',
	},
	userColumn: {
		id: 'servers.audit-log.column.user',
		defaultMessage: 'User',
	},
	worldColumn: {
		id: 'servers.audit-log.column.world',
		defaultMessage: 'World',
	},
	actionColumn: {
		id: 'servers.audit-log.column.action',
		defaultMessage: 'Action',
	},
	timeColumn: {
		id: 'servers.audit-log.column.time',
		defaultMessage: 'Time',
	},
	emptyState: {
		id: 'servers.audit-log.empty',
		defaultMessage: 'No activity matches your filters.',
	},
	userAvatarAlt: {
		id: 'servers.audit-log.user-avatar-alt',
		defaultMessage: "{username}'s avatar",
	},
	contentIconAlt: {
		id: 'servers.audit-log.content-icon-alt',
		defaultMessage: "{name}'s icon",
	},
	modInstalledPrefix: {
		id: 'servers.audit-log.action-prefix.mod-installed',
		defaultMessage: 'Installed mod:',
	},
	modpackInstalledPrefix: {
		id: 'servers.audit-log.action-prefix.modpack-installed',
		defaultMessage: 'Installed modpack:',
	},
	memberInvitedPrefix: {
		id: 'servers.audit-log.action-prefix.member-invited',
		defaultMessage: 'Invited user:',
	},
	memberRemovedPrefix: {
		id: 'servers.audit-log.action-prefix.member-removed',
		defaultMessage: 'Removed user:',
	},
	roleChangedPrefix: {
		id: 'servers.audit-log.action-prefix.role-changed',
		defaultMessage: 'Changed role:',
	},
	memberInvitedRoleSuffix: {
		id: 'servers.audit-log.action-suffix.member-invited-role',
		defaultMessage: 'as {role}',
	},
	roleChangedRoleSuffix: {
		id: 'servers.audit-log.action-suffix.role-changed-role',
		defaultMessage: 'to {role}',
	},
	fileEditedAction: {
		id: 'servers.audit-log.action.file-edited',
		defaultMessage: 'Edited file: {file}',
	},
	worldStartedAction: {
		id: 'servers.audit-log.action.world-started',
		defaultMessage: 'Started world: {worldName}',
	},
	modInstalledAction: {
		id: 'servers.audit-log.action.mod-installed',
		defaultMessage: 'Installed mod: {name}{version}',
	},
	modpackInstalledAction: {
		id: 'servers.audit-log.action.modpack-installed',
		defaultMessage: 'Installed modpack: {name}{version}',
	},
	memberInvitedAction: {
		id: 'servers.audit-log.action.member-invited',
		defaultMessage: 'Invited user: {target}{role}',
	},
	memberRemovedAction: {
		id: 'servers.audit-log.action.member-removed',
		defaultMessage: 'Removed user: {target}',
	},
	roleChangedAction: {
		id: 'servers.audit-log.action.role-changed',
		defaultMessage: 'Changed role: {target}{role}',
	},
	ownerRole: {
		id: 'servers.audit-log.role.owner',
		defaultMessage: 'Owner',
	},
	editorRole: {
		id: 'servers.audit-log.role.editor',
		defaultMessage: 'Editor',
	},
	viewerRole: {
		id: 'servers.audit-log.role.viewer',
		defaultMessage: 'Limited',
	},
})

const timeRangeOptions = computed<ComboboxOption<AuditTimeRange>[]>(() => [
	{ value: 'previous_30_minutes', label: formatMessage(messages.previous30Minutes) },
	{ value: 'previous_hour', label: formatMessage(messages.previousHour) },
	{ value: 'previous_12_hours', label: formatMessage(messages.previous12Hours) },
	{ value: 'previous_24_hours', label: formatMessage(messages.previous24Hours) },
	{ value: 'today', label: formatMessage(messages.today) },
	{ value: 'yesterday', label: formatMessage(messages.yesterday) },
	{ value: 'this_week', label: formatMessage(messages.thisWeek) },
	{ value: 'last_week', label: formatMessage(messages.lastWeek) },
	{ value: 'previous_7_days', label: formatMessage(messages.previous7Days) },
	{ value: 'this_month', label: formatMessage(messages.thisMonth) },
	{ value: 'last_month', label: formatMessage(messages.lastMonth) },
	{ value: 'last_30_days', label: formatMessage(messages.last30Days) },
	{ value: 'this_quarter', label: formatMessage(messages.thisQuarter) },
	{ value: 'last_quarter', label: formatMessage(messages.lastQuarter) },
	{ value: 'this_year', label: formatMessage(messages.thisYear) },
	{ value: 'last_year', label: formatMessage(messages.lastYear) },
	{ value: 'previous_year', label: formatMessage(messages.previousYear) },
	{ value: 'previous_two_years', label: formatMessage(messages.previousTwoYears) },
	{ value: 'all_time', label: formatMessage(messages.allTime) },
])

const selectedTimeRangeLabel = computed(
	() =>
		timeRangeOptions.value.find((option) => option.value === selectedTimeRange.value)?.label ??
		formatMessage(messages.last30Days),
)

type AuditLogTableColumn = 'user' | 'world' | 'action' | 'time'
type AuditLogTableRow = ServerAuditLogEntry & Record<string, unknown>
type MemberAuditAction = Extract<
	ServerAuditAction,
	{ type: 'member_invited' | 'member_removed' | 'role_changed' }
>
type ActionLabelViewport = 'desktop' | 'mobile'

const actionLabelRefs = shallowReactive<Record<string, HTMLElement | null>>({})

const columns = computed<TableColumn<AuditLogTableColumn>[]>(() => [
	{ key: 'user', label: formatMessage(messages.userColumn), width: '20%' },
	{ key: 'world', label: formatMessage(messages.worldColumn), width: '18%' },
	{ key: 'action', label: formatMessage(messages.actionColumn), width: '44%' },
	{ key: 'time', label: formatMessage(messages.timeColumn), align: 'right', width: '18%' },
])

const filteredEntries = computed(() => {
	const normalizedQuery = query.value.trim().toLowerCase()

	return props.entries
		.filter((entry) => {
			if (filters.value.userId && entry.actor.id !== filters.value.userId) return false
			if (filters.value.worldId && entry.world?.id !== filters.value.worldId) return false
			if (filters.value.actionType && entry.action.type !== filters.value.actionType) return false

			if (!normalizedQuery) return true

			return [
				entry.actor.username,
				entry.world?.name,
				formatAction(entry.action),
				actionSearchValue(entry.action),
			]
				.filter(Boolean)
				.some((value) => value!.toLowerCase().includes(normalizedQuery))
		})
		.slice()
		.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
})

const tableEntries = computed<AuditLogTableRow[]>(() => filteredEntries.value as AuditLogTableRow[])

function formatAction(action: ServerAuditAction): string {
	switch (action.type) {
		case 'file_edited':
			return formatMessage(messages.fileEditedAction, { file: action.file })
		case 'world_started':
			return formatMessage(messages.worldStartedAction, { worldName: action.worldName })
		case 'content_installed':
			return action.contentType === 'mod'
				? formatMessage(messages.modInstalledAction, {
						name: action.name,
						version: formatVersionSuffix(action.version),
					})
				: formatMessage(messages.modpackInstalledAction, {
						name: action.name,
						version: formatVersionSuffix(action.version),
					})
		case 'member_invited':
			return formatMessage(messages.memberInvitedAction, {
				target: action.target,
				role: action.role ? ` as ${formatRole(action.role)}` : '',
			})
		case 'member_removed':
			return formatMessage(messages.memberRemovedAction, { target: action.target })
		case 'role_changed':
			return formatMessage(messages.roleChangedAction, {
				target: action.target,
				role: action.role ? ` to ${formatRole(action.role)}` : '',
			})
	}
}

function actionSearchValue(action: ServerAuditAction): string {
	switch (action.type) {
		case 'file_edited':
			return action.file
		case 'world_started':
			return action.worldName
		case 'content_installed':
			return `${action.contentType} ${action.name} ${action.version ?? ''}`
		case 'member_invited':
			return `${action.target} ${action.role ? formatRole(action.role) : ''}`
		case 'member_removed':
			return action.target
		case 'role_changed':
			return `${action.target} ${action.role ? formatRole(action.role) : ''}`
	}
}

function contentInstalledPrefix(action: ServerAuditAction): string {
	if (action.type !== 'content_installed') return ''
	return action.contentType === 'mod'
		? formatMessage(messages.modInstalledPrefix)
		: formatMessage(messages.modpackInstalledPrefix)
}

function isMemberAuditAction(action: ServerAuditAction): action is MemberAuditAction {
	return (
		action.type === 'member_invited' ||
		action.type === 'member_removed' ||
		action.type === 'role_changed'
	)
}

function memberActionPrefix(action: MemberAuditAction): string {
	switch (action.type) {
		case 'member_invited':
			return formatMessage(messages.memberInvitedPrefix)
		case 'member_removed':
			return formatMessage(messages.memberRemovedPrefix)
		case 'role_changed':
			return formatMessage(messages.roleChangedPrefix)
		default:
			return ''
	}
}

function memberActionSuffix(action: MemberAuditAction): string {
	switch (action.type) {
		case 'member_invited':
			return action.role
				? formatMessage(messages.memberInvitedRoleSuffix, { role: formatRole(action.role) })
				: ''
		case 'role_changed':
			return action.role
				? formatMessage(messages.roleChangedRoleSuffix, { role: formatRole(action.role) })
				: ''
		default:
			return ''
	}
}

function actionLabelRefKey(entryId: string, viewport: ActionLabelViewport): string {
	return `${viewport}-${entryId}`
}

function setActionLabelRef(key: string, element: unknown) {
	actionLabelRefs[key] = element instanceof HTMLElement ? element : null
}

function actionTooltip(
	entry: ServerAuditLogEntry,
	viewport: ActionLabelViewport,
): string | undefined {
	return truncatedTooltip(
		actionLabelRefs[actionLabelRefKey(entry.id, viewport)],
		formatAction(entry.action),
	)
}

function actorName(entry: ServerAuditLogEntry): string {
	return entry.actor.id === 'support' ? formatMessage(messages.supportActor) : entry.actor.username
}

function actorAvatarSrc(entry: ServerAuditLogEntry): string | undefined {
	return entry.actor.id === 'support' ? IntercomBubbleIcon : (entry.actor.avatarUrl ?? undefined)
}

function actorProfilePath(entry: ServerAuditLogEntry): string | undefined {
	return entry.actor.id === 'support' ? undefined : userProfilePath(entry.actor.username)
}

function targetAvatarSrc(username: string): string | undefined {
	return props.users.find((member) => member.user.username === username)?.user.avatarUrl
}

function actionLinkClass(href: string | undefined): string {
	return href ? 'text-secondary hover:text-primary hover:underline' : 'text-primary'
}

function userProfilePath(username: string): string | undefined {
	if (!username || username.includes('@')) return undefined
	return `/user/${encodeURIComponent(username)}`
}

function formatVersionSuffix(version: string | undefined): string {
	return version ? ` (${version})` : ''
}

function formatRole(role: ServerAccessRole): string {
	switch (role) {
		case 'owner':
			return formatMessage(messages.ownerRole)
		case 'editor':
			return formatMessage(messages.editorRole)
		case 'viewer':
			return formatMessage(messages.viewerRole)
	}
}
</script>
