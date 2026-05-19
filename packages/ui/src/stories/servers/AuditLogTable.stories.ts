import type { Archon } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import DropdownFilterBar from '../../components/base/DropdownFilterBar.vue'
import AuditLogTable from '../../components/servers/access/AuditLogTable.vue'
import {
	parseAuditEvent,
	type AuditEventLookups,
} from '../../components/servers/access/events'
import type {
	ServerAuditLogEntry,
	ServerAuditLogFilters,
} from '../../components/servers/access/types'

const serverId = 'story-server'
const createWorldId = 'create-smp'
const activeWorldId = 'smp-season-4'
const backupId = '00000000-0000-4000-8000-000000000001'

const userIds = {
	geometrically: 'MpxzqsyW',
	modmuss: 'JZA4dW8o',
	prospector: 'Dc7EYhxG',
}

const filterBarCategories = [
	{
		key: 'users',
		label: 'Users',
		searchable: true,
		searchPlaceholder: 'Search users...',
		options: [
			{ value: userIds.geometrically, label: 'Geometrically' },
			{ value: userIds.modmuss, label: 'modmuss50' },
			{ value: userIds.prospector, label: 'Prospector' },
		],
	},
	{
		key: 'worlds',
		label: 'Instances',
		options: [
			{ value: createWorldId, label: 'Create SMP' },
			{ value: activeWorldId, label: 'SMP Season 4' },
		],
	},
	{
		key: 'actions',
		label: 'Action types',
		options: [
			{ value: 'server_started', label: 'Server started' },
			{ value: 'console_command_executed', label: 'Console command run' },
			{ value: 'backup_restored', label: 'Backup restored' },
		],
	},
]

const serverFullResponse: Archon.Servers.v1.ServerFull = {
	id: serverId,
	name: 'Stoneblock Lab',
	subdomain: 'stoneblock-lab',
	specs: {
		cpu: 4,
		memory_mb: 8192,
		storage_mb: 65536,
		swap_mb: 1024,
	},
	sftp_username: 'story-server',
	sftp_password: 'temporary-password',
	tags: ['medal'],
	location: {
		status: 'assigned',
		location_metadata: {
			region: 'us-east',
			region_should_be_user_displayed: true,
			hostname: 'ashburn-01',
			is_decommissioned_node: false,
		},
	},
	worlds: [
		{
			id: createWorldId,
			name: 'Create SMP',
			created_at: new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString(),
			is_active: false,
			backups: [],
			content: {
				modloader: 'fabric',
				modloader_version: '0.16.10',
				game_version: '1.21.1',
				java_version: 21,
				invocation: 'java -Xmx8G -jar server.jar nogui',
				original_invocation: 'java -jar server.jar nogui',
			},
			readiness: {
				data_synchronized_fetched: true,
			},
		},
		{
			id: activeWorldId,
			name: 'SMP Season 4',
			created_at: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
			is_active: true,
			backups: [
				{
					id: backupId,
					physical_id: backupId,
					name: 'Before modpack update',
					created_at: new Date(Date.now() - 8 * 60 * 60 * 1000).toISOString(),
					automated: false,
					status: 'done',
					interrupted: false,
					ongoing: false,
					locked: false,
				},
			],
			content: {
				modloader: 'fabric',
				modloader_version: '0.16.10',
				game_version: '1.21.1',
				java_version: 21,
				invocation: 'java -Xmx8G -jar server.jar nogui',
				original_invocation: 'java -jar server.jar nogui',
			},
			readiness: {
				data_synchronized_fetched: true,
			},
		},
	],
}

const actionLogResponse: Archon.Actions.v1.ActionLogResponse = {
	next_offset: null,
	users: {
		[userIds.geometrically]: {
			username: 'Geometrically',
			avatar_url:
				'https://cdn.modrinth.com/data/MpxzqsyW/eb0038489a55e7e7a188a5b50462f0b10dfc1613_96.webp',
		},
		[userIds.modmuss]: {
			username: 'modmuss50',
			avatar_url: 'https://avatars2.githubusercontent.com/u/4324090?v=4',
		},
		[userIds.prospector]: {
			username: 'Prospector',
			avatar_url:
				'https://cdn.modrinth.com/user/Dc7EYhxG/32e8b1f7d18288262d1ed92cbdf43272d21b4fcd.png',
		},
	},
	addons: {
		oWaK0Q19: {
			title: 'Create Aeronautics',
			slug: 'create-aeronautics',
			icon_url: 'https://cdn.modrinth.com/data/oWaK0Q19/f66b5589924884ffd81acb27f3ccb775867a962e_96.webp',
			version: null,
		},
		AANobbMI: {
			title: 'Sodium',
			slug: 'sodium',
			icon_url: 'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',
			version: null,
		},
		P7dR8mSH: {
			title: 'Fabric API',
			slug: 'fabric-api',
			icon_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
			version: null,
		},
	},
	data: [
		rawEntry({ action: 'server_created', worldId: null, minutesAgo: 5 }),
		rawEntry({
			action: 'changed_server_name',
			metadata: { name: 'Stoneblock Lab' },
			worldId: null,
			minutesAgo: 10,
		}),
		rawEntry({
			action: 'changed_server_subdomain',
			metadata: { subdomain: 'stoneblock-lab' },
			worldId: null,
			minutesAgo: 15,
		}),
		rawEntry({ action: 'server_reallocated', worldId: null, minutesAgo: 20 }),
		rawEntry({
			action: 'server_plan_changed',
			metadata: { new_specs: { cpu: 4, memory_mb: 8192, storage_mb: 65536, swap_mb: 1024 } },
			worldId: null,
			minutesAgo: 25,
		}),
		rawEntry({
			action: 'user_invited',
			metadata: { user_id: userIds.modmuss, permissions: 'BASE_READ | POWER_ACTIONS' },
			worldId: null,
			minutesAgo: 30,
		}),
		rawEntry({
			action: 'user_invite_revoked',
			metadata: { user_id: userIds.modmuss },
			worldId: null,
			minutesAgo: 35,
		}),
		rawEntry({
			action: 'user_permission_modified',
			metadata: {
				user_id: userIds.modmuss,
				permissions: 'BASE_READ | POWER_ACTIONS | FILES_WRITE | SETUP | BACKUPS | ADVANCED',
			},
			worldId: null,
			minutesAgo: 40,
		}),
		rawEntry({
			action: 'user_removed',
			metadata: { user_id: userIds.modmuss },
			worldId: null,
			minutesAgo: 45,
		}),
		rawEntry({
			action: 'addon_added',
			metadata: { addons: [{ addon_id: 'oWaK0Q19', version_id: 'HY8u0JqC' }] },
			minutesAgo: 50,
		}),
		rawEntry({
			action: 'addon_uploaded',
			metadata: { file_names: ['custom-tweaks.jar', 'server-rules.datapack.zip'] },
			minutesAgo: 55,
		}),
		rawEntry({
			action: 'addon_disabled',
			metadata: { addons: [{ addon_id: 'AANobbMI', version_id: 'yaoBL9D9' }] },
			minutesAgo: 60,
		}),
		rawEntry({
			action: 'addon_enabled',
			metadata: { addons: [{ addon_id: 'AANobbMI', version_id: 'yaoBL9D9' }] },
			minutesAgo: 65,
		}),
		rawEntry({
			action: 'addon_deleted',
			metadata: { addons: [{ addon_id: 'P7dR8mSH', version_id: 'KZS9tylY' }] },
			minutesAgo: 70,
		}),
		rawEntry({
			action: 'addon_updated',
			metadata: {
				addons: [
					{ addon_id: 'AANobbMI', version_id: 'YAGZ1cCS' },
					{ addon_id: 'P7dR8mSH', version_id: 'EW33COvi' },
				],
			},
			minutesAgo: 75,
		}),
		rawEntry({
			action: 'modpack_changed',
			metadata: { new_version: 'version-modpack-001' },
			minutesAgo: 80,
		}),
		rawEntry({ action: 'modpack_unlinked', minutesAgo: 85 }),
		rawEntry({ action: 'server_repaired', minutesAgo: 90 }),
		rawEntry({ action: 'server_reset', minutesAgo: 95 }),
		rawEntry({ action: 'server_started', minutesAgo: 100 }),
		rawEntry({ action: 'server_stopped', minutesAgo: 105 }),
		rawEntry({ action: 'server_restarted', minutesAgo: 110 }),
		rawEntry({ action: 'server_killed', minutesAgo: 115 }),
		rawEntry({
			action: 'port_allocation_added',
			metadata: { port: 25565 },
			worldId: null,
			minutesAgo: 120,
		}),
		rawEntry({
			action: 'port_allocation_removed',
			metadata: { port: 24454 },
			worldId: null,
			minutesAgo: 125,
		}),
		rawEntry({
			action: 'loader_version_edited',
			metadata: { new_version: '0.16.10' },
			minutesAgo: 130,
		}),
		rawEntry({
			action: 'game_version_edited',
			metadata: { new_version: '1.21.1' },
			minutesAgo: 135,
		}),
		rawEntry({
			action: 'server_properties_modified',
			metadata: { properties: { difficulty: 'hard', 'max-players': '20' } },
			minutesAgo: 140,
		}),
		rawEntry({ action: 'file_uploaded', metadata: { path: '/mods/custom.jar' }, minutesAgo: 145 }),
		rawEntry({ action: 'file_deleted', metadata: { path: '/config/old.toml' }, minutesAgo: 150 }),
		rawEntry({
			action: 'file_renamed',
			metadata: { from: '/world/old.dat', to: '/world/new.dat' },
			minutesAgo: 155,
		}),
		rawEntry({ action: 'file_edited', metadata: { path: '/server.properties' }, minutesAgo: 160 }),
		rawEntry({ action: 'sftp_login', minutesAgo: 165 }),
		rawEntry({
			action: 'console_command_executed',
			metadata: { command: 'whitelist add Prospector' },
			minutesAgo: 170,
		}),
		rawEntry({ action: 'console_cleared', minutesAgo: 175 }),
		rawEntry({
			action: 'backup_created',
			metadata: { id: backupId },
			minutesAgo: 180,
		}),
		rawEntry({
			action: 'backup_renamed',
			metadata: {
				id: backupId,
				from: 'Manual backup 1',
				to: 'Before modpack update',
			},
			minutesAgo: 185,
		}),
		rawEntry({
			action: 'backup_restored',
			metadata: { id: backupId },
			minutesAgo: 190,
		}),
		rawEntry({
			action: 'backup_deleted',
			metadata: { id: '00000000-0000-4000-8000-000000000099' },
			minutesAgo: 195,
		}),
		rawEntry({
			action: 'startup_command_modified',
			metadata: { command: 'java -Xmx8G -jar server.jar nogui' },
			minutesAgo: 200,
		}),
		rawEntry({
			action: 'java_runtime_modified',
			metadata: { vendor: 'temurin' },
			minutesAgo: 205,
		}),
		rawEntry({
			action: 'java_version_modified',
			metadata: { version: 21 },
			minutesAgo: 210,
		}),
	],
}

const missingLookupActionLogResponse: Archon.Actions.v1.ActionLogResponse = {
	next_offset: null,
	users: {},
	addons: {},
	data: [
		rawEntry({
			action: 'user_permission_modified',
			metadata: { user_id: 'unknown-user', permissions: 'BASE_READ' },
			worldId: 'unknown-world',
			minutesAgo: 5,
		}),
		rawEntry({
			action: 'addon_updated',
			metadata: { addons: [{ addon_id: 'unknown-addon', version_id: 'unknown-version' }] },
			minutesAgo: 10,
		}),
		rawEntry({
			action: 'modpack_changed',
			metadata: { new_version: null },
			minutesAgo: 15,
		}),
		rawEntry({
			action: 'backup_deleted',
			metadata: { id: '00000000-0000-4000-8000-000000000099' },
			minutesAgo: 20,
		}),
		rawEntry({
			action: 'changed_server_name',
			metadata: { bad: true },
			minutesAgo: 25,
		}),
		rawEntry({
			action: 'backend_added_this_later',
			metadata: { value: true },
			actor: { type: 'support', user_id: userIds.prospector },
			worldId: null,
			minutesAgo: 30,
		}),
	],
}

type RawEntryInput = {
	action: Archon.Actions.v1.ActionName | string
	metadata?: unknown
	actor?: Archon.Actions.v1.ActionUser
	worldId?: string | null
	minutesAgo: number
}

function rawEntry(input: RawEntryInput): Archon.Actions.v1.ActionEntry {
	return {
		actor: input.actor ?? { type: 'user', user_id: userIds.geometrically },
		action:
			input.metadata === undefined
				? { action: input.action }
				: { action: input.action, metadata: input.metadata },
		server_id: serverId,
		world_id: input.worldId ?? activeWorldId,
		timestamp: new Date(Date.now() - input.minutesAgo * 60 * 1000).toISOString(),
	}
}

function lookupsFromResponse(
	response: Archon.Actions.v1.ActionLogResponse,
	serverFull: Archon.Servers.v1.ServerFull,
): AuditEventLookups {
	const backups = new Map<string, Archon.Backups.v1.Backup>()
	for (const world of serverFull.worlds) {
		for (const backup of world.backups) {
			backups.set(backup.id, backup)
		}
	}

	return {
		serverId: serverFull.id,
		users: response.users,
		addons: response.addons,
		worldById: new Map(
			serverFull.worlds.map((world) => [world.id, { id: world.id, name: world.name }]),
		),
		backupById: backups,
		versions: undefined,
	}
}

function toAuditEntries(
	response: Archon.Actions.v1.ActionLogResponse,
	serverFull = serverFullResponse,
): ServerAuditLogEntry[] {
	const lookups = lookupsFromResponse(response, serverFull)
	return response.data.map((entry, index) => {
		const event = parseAuditEvent(entry, lookups)
		return {
			id: `${entry.timestamp}-${entry.actor.type}-${entry.world_id ?? 'server'}-${index}`,
			actor: event.props.actor,
			world: event.props.world,
			event,
			timestamp: entry.timestamp,
		}
	})
}

const everyActionEntries = toAuditEntries(actionLogResponse)
const fallbackEntries = toAuditEntries(missingLookupActionLogResponse, {
	...serverFullResponse,
	worlds: [],
})

const meta = {
	title: 'Servers/AuditLogTable',
	component: AuditLogTable,
	parameters: {
		layout: 'padded',
		docs: {
			description: {
				component: 'Audit entries render backend action-log events through typed event components.',
			},
		},
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 1120px;"><story /></div>',
		}),
	],
} satisfies Meta<typeof AuditLogTable>

export default meta
type Story = StoryObj<typeof meta>

function renderStory(entries: ServerAuditLogEntry[], initialQuery = '') {
	return () => ({
		components: { AuditLogTable },
		setup() {
			const query = ref(initialQuery)
			const filters = ref<ServerAuditLogFilters>({
				userId: null,
				worldId: null,
			})
			return { entries, query, filters }
		},
		template: /* html */ `
			<AuditLogTable
				v-model:query="query"
				v-model:filters="filters"
				:entries="entries"
			/>
		`,
	})
}

export const AllEvents: Story = {
	render: renderStory(everyActionEntries),
}

export const MissingLookupsAndFallbacks: Story = {
	render: renderStory(fallbackEntries),
}

export const Filtered: Story = {
	render: renderStory(everyActionEntries, 'Create Aeronautics'),
}

export const WithExternalFilterControls: Story = {
	render: () => ({
		components: { AuditLogTable, DropdownFilterBar },
		setup() {
			const query = ref('')
			const filters = ref<ServerAuditLogFilters>({
				userId: null,
				worldId: null,
			})
			const externalFilters = ref<Record<string, string[]>>({
				users: [userIds.geometrically],
				worlds: [],
				actions: [],
			})
			return {
				categories: filterBarCategories,
				entries: everyActionEntries,
				externalFilters,
				filters,
				query,
			}
		},
		template: /* html */ `
			<AuditLogTable
				v-model:query="query"
				v-model:filters="filters"
				:entries="entries"
				has-active-external-filters
			>
				<template #filters>
					<DropdownFilterBar
						v-model="externalFilters"
						:categories="categories"
						add-label="Add filter"
						clear-label="Clear"
						use-filter-icon
					/>
				</template>
			</AuditLogTable>
		`,
	}),
}

export const EmptyExternalFilters: Story = {
	render: () => ({
		components: { AuditLogTable },
		setup() {
			const query = ref('')
			const filters = ref<ServerAuditLogFilters>({
				userId: null,
				worldId: null,
			})
			return { entries: [], query, filters }
		},
		template: /* html */ `
			<AuditLogTable
				v-model:query="query"
				v-model:filters="filters"
				:entries="entries"
				has-active-external-filters
			/>
		`,
	}),
}

export const MobileCompact: Story = {
	render: () => ({
		components: { AuditLogTable },
		setup() {
			const query = ref('')
			const filters = ref<ServerAuditLogFilters>({
				userId: null,
				worldId: null,
			})
			return { entries: everyActionEntries.slice(0, 8), query, filters }
		},
		template: /* html */ `
			<div style="max-width: 390px;">
				<AuditLogTable
					v-model:query="query"
					v-model:filters="filters"
					:entries="entries"
				/>
			</div>
		`,
	}),
}
