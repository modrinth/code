import type { Archon } from '@modrinth/api-client'
import type { IconComponent } from '@modrinth/assets'
import {
	DatabaseBackupIcon,
	FileIcon,
	PackageIcon,
	PowerIcon,
	ServerIcon,
	SettingsIcon,
	UsersIcon,
} from '@modrinth/assets'

import type { DropdownFilterBarOption } from '#ui/components/base/DropdownFilterBar.vue'
import type {
	TimeFrameLastUnit,
	TimeFrameMode,
	TimeFramePreset,
} from '#ui/components/base/TimeFramePicker.vue'
import { defineMessage, type MessageDescriptor } from '#ui/composables/i18n'

export const SUPPORT_ACTION_LOG_USER_FILTER = 'support'
export const SERVER_SCOPED_ACTION_LOG_WORLD_FILTER = '__server_scoped__'

export const actionLogActionNames = [
	'server_created',
	'changed_server_name',
	'changed_server_subdomain',
	'server_reallocated',
	'server_plan_changed',
	'user_invited',
	'user_invite_revoked',
	'user_permission_modified',
	'user_removed',
	'addon_added',
	'addon_uploaded',
	'addon_disabled',
	'addon_enabled',
	'addon_deleted',
	'addon_updated',
	'modpack_changed',
	'modpack_unlinked',
	'server_repaired',
	'server_reset',
	'server_started',
	'server_stopped',
	'server_restarted',
	'server_killed',
	'port_allocation_added',
	'port_allocation_removed',
	'loader_version_edited',
	'game_version_edited',
	'server_properties_modified',
	'file_uploaded',
	'file_deleted',
	'file_renamed',
	'file_edited',
	'sftp_login',
	'console_command_executed',
	'console_cleared',
	'backup_created',
	'backup_renamed',
	'backup_restored',
	'backup_deleted',
	'startup_command_modified',
	'java_runtime_modified',
	'java_version_modified',
] as const satisfies readonly Archon.Actions.v1.ActionName[]

export type ActionLogFilterActionName = (typeof actionLogActionNames)[number]

const actionLogActionNameSet = new Set<string>(actionLogActionNames)

export const actionLogActionGroups = [
	{
		key: 'server',
		label: defineMessage({
			id: 'servers.access-page.activity-log-filter.action-group.server',
			defaultMessage: 'Server',
		}),
		icon: ServerIcon,
		actions: [
			'server_created',
			'server_reallocated',
			'server_plan_changed',
			'server_repaired',
			'server_reset',
		],
	},
	{
		key: 'power-console',
		label: defineMessage({
			id: 'servers.access-page.activity-log-filter.action-group.power-console',
			defaultMessage: 'Power and console',
		}),
		icon: PowerIcon,
		actions: [
			'server_started',
			'server_stopped',
			'server_restarted',
			'server_killed',
			'console_command_executed',
			'console_cleared',
		],
	},
	{
		key: 'users',
		label: defineMessage({
			id: 'servers.access-page.activity-log-filter.action-group.users',
			defaultMessage: 'Users and invites',
		}),
		icon: UsersIcon,
		actions: ['user_invited', 'user_invite_revoked', 'user_permission_modified', 'user_removed'],
	},
	{
		key: 'content',
		label: defineMessage({
			id: 'servers.access-page.activity-log-filter.action-group.content',
			defaultMessage: 'Content and modpack',
		}),
		icon: PackageIcon,
		actions: [
			'addon_added',
			'addon_uploaded',
			'addon_disabled',
			'addon_enabled',
			'addon_updated',
			'addon_deleted',
			'modpack_changed',
			'modpack_unlinked',
		],
	},
	{
		key: 'files',
		label: defineMessage({
			id: 'servers.access-page.activity-log-filter.action-group.files',
			defaultMessage: 'Files and SFTP',
		}),
		icon: FileIcon,
		actions: ['file_uploaded', 'file_edited', 'file_renamed', 'file_deleted', 'sftp_login'],
	},
	{
		key: 'backups',
		label: defineMessage({
			id: 'servers.access-page.activity-log-filter.action-group.backups',
			defaultMessage: 'Backups',
		}),
		icon: DatabaseBackupIcon,
		actions: ['backup_created', 'backup_renamed', 'backup_restored', 'backup_deleted'],
	},
	{
		key: 'settings',
		label: defineMessage({
			id: 'servers.access-page.activity-log-filter.action-group.settings',
			defaultMessage: 'Settings and runtime',
		}),
		icon: SettingsIcon,
		actions: [
			'changed_server_name',
			'changed_server_subdomain',
			'port_allocation_added',
			'port_allocation_removed',
			'loader_version_edited',
			'game_version_edited',
			'server_properties_modified',
			'startup_command_modified',
			'java_runtime_modified',
			'java_version_modified',
		],
	},
] as const satisfies readonly {
	key: string
	label: MessageDescriptor
	icon: IconComponent
	actions: readonly ActionLogFilterActionName[]
}[]

export type AuditLogTimeframeSelection = {
	mode: TimeFrameMode
	preset: TimeFramePreset
	lastAmount: number
	lastUnit: TimeFrameLastUnit
	customStartDate: string
	customEndDate: string
}

export function isActionLogActionName(action: string): action is ActionLogFilterActionName {
	return actionLogActionNameSet.has(action)
}

export function compareFilterOptions(
	left: DropdownFilterBarOption,
	right: DropdownFilterBarOption,
) {
	return left.label.localeCompare(right.label)
}

export function getAuditLogTimeframeRange(
	selection: AuditLogTimeframeSelection,
): { start: Date; end: Date } | null {
	const now = getRoundedNow()

	if (selection.mode === 'last') {
		return getLastAuditLogTimeframeRange(selection.lastAmount, selection.lastUnit, now)
	}

	if (selection.mode === 'custom_range') {
		const startDate = parseDateInputValue(selection.customStartDate)
		const endDate = parseDateInputValue(selection.customEndDate)
		if (!startDate || !endDate) return null

		const [minDate, maxDate] =
			startDate.getTime() > endDate.getTime() ? [endDate, startDate] : [startDate, endDate]

		return {
			start: startOfDay(minDate),
			end: endOfDay(maxDate),
		}
	}

	if (selection.mode !== 'preset') {
		return null
	}

	return getPresetAuditLogTimeframeRange(selection.preset, now)
}

export function getActionLogEntryId(entry: Archon.Actions.v1.ActionEntry) {
	return JSON.stringify([
		entry.timestamp,
		entry.actor.type,
		entry.actor.type === 'user' ? entry.actor.user_id : (entry.actor.user_id ?? 'support'),
		entry.server_id,
		entry.world_id ?? null,
		entry.action.action,
		stableStringify(entry.action.metadata),
	])
}

function parseDateInputValue(value: string) {
	const [yearValue, monthValue, dayValue] = value.split('-').map(Number)
	if (!yearValue || !monthValue || !dayValue) return null

	const date = new Date(yearValue, monthValue - 1, dayValue)
	if (
		date.getFullYear() !== yearValue ||
		date.getMonth() !== monthValue - 1 ||
		date.getDate() !== dayValue
	) {
		return null
	}

	return date
}

function addDays(date: Date, days: number) {
	const nextDate = new Date(date)
	nextDate.setDate(nextDate.getDate() + days)
	return nextDate
}

function subtractCalendarMonths(date: Date, months: number) {
	const nextDate = new Date(date)
	const day = nextDate.getDate()
	nextDate.setDate(1)
	nextDate.setMonth(nextDate.getMonth() - months)
	const daysInMonth = new Date(nextDate.getFullYear(), nextDate.getMonth() + 1, 0).getDate()
	nextDate.setDate(Math.min(day, daysInMonth))
	return nextDate
}

function getRoundedNow() {
	const now = Date.now()
	return new Date(Math.floor(now / 60000) * 60000)
}

function getPresetAuditLogTimeframeRange(
	preset: TimeFramePreset,
	now: Date,
): { start: Date; end: Date } | null {
	switch (preset) {
		case 'today':
			return { start: startOfDay(now), end: endOfDay(now) }
		case 'yesterday': {
			const yesterday = addDays(now, -1)
			return { start: startOfDay(yesterday), end: endOfDay(yesterday) }
		}
		case 'last_7_days':
			return { start: startOfDay(addDays(now, -6)), end: endOfDay(now) }
		case 'last_14_days':
			return { start: startOfDay(addDays(now, -13)), end: endOfDay(now) }
		case 'last_30_days':
			return { start: startOfDay(addDays(now, -29)), end: endOfDay(now) }
		case 'last_90_days':
			return { start: startOfDay(addDays(now, -89)), end: endOfDay(now) }
		case 'last_180_days':
			return { start: startOfDay(addDays(now, -179)), end: endOfDay(now) }
		case 'year_to_date':
			return { start: new Date(now.getFullYear(), 0, 1), end: endOfDay(now) }
		case 'all_time':
			return null
	}
}

function getLastAuditLogTimeframeRange(
	amountValue: number,
	unit: TimeFrameLastUnit,
	now: Date,
): { start: Date; end: Date } {
	const amount = Math.max(1, Math.floor(amountValue))

	switch (unit) {
		case 'hours':
			return { start: new Date(now.getTime() - amount * 60 * 60 * 1000), end: now }
		case 'days':
			return { start: new Date(now.getTime() - amount * 24 * 60 * 60 * 1000), end: now }
		case 'weeks':
			return { start: new Date(now.getTime() - amount * 7 * 24 * 60 * 60 * 1000), end: now }
		case 'months':
			return { start: subtractCalendarMonths(now, amount), end: now }
	}
}

function startOfDay(date: Date) {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate())
}

function endOfDay(date: Date) {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate(), 23, 59, 59, 999)
}

function stableStringify(value: unknown): string {
	if (value === undefined) {
		return 'undefined'
	}

	if (value === null || typeof value !== 'object') {
		return JSON.stringify(value) ?? String(value)
	}

	if (Array.isArray(value)) {
		return `[${value.map((item) => stableStringify(item)).join(',')}]`
	}

	return `{${Object.entries(value as Record<string, unknown>)
		.sort(([leftKey], [rightKey]) => leftKey.localeCompare(rightKey))
		.map(([key, item]) => `${JSON.stringify(key)}:${stableStringify(item)}`)
		.join(',')}}`
}
