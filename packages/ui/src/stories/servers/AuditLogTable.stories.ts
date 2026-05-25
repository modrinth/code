import type { Archon } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import auditLogEntryExamplesJson from './audit-log-entry-examples.json?raw'
import DropdownFilterBar from '../../components/base/DropdownFilterBar.vue'
import type {
	TimeFrameLastUnit,
	TimeFrameMode,
	TimeFramePreset,
} from '../../components/base/TimeFramePicker.vue'
import AuditLogTable from '../../components/servers/access/AuditLogTable.vue'
import { type AuditEventLookups, parseAuditEvent } from '../../components/servers/access/events'
import type {
	ServerAuditLogEntry,
	ServerAuditLogFilters,
} from '../../components/servers/access/types'

const serverId = 'story-server'
const createWorldId = 'create-smp'
const activeWorldId = 'smp-season-4'
const backupId = '00000000-0000-4000-8000-000000000001'

type AuditLogEntryExamplesFixture = Archon.Actions.v1.ActionLogResponse & {
	server_id: string
	server: {
		id: string
		name: string
		subdomain: string
	}
	worlds: Array<{
		id: string
		name: string
	}>
	backups: Archon.Backups.v1.Backup[]
}

const auditLogEntryExamples = JSON.parse(auditLogEntryExamplesJson) as AuditLogEntryExamplesFixture

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
			{ value: 'server_started', label: 'Started server' },
			{ value: 'console_command_executed', label: 'Ran console command' },
			{ value: 'backup_restored', label: 'Restored backup' },
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

const auditLogEntryExamplesServerFullResponse: Archon.Servers.v1.ServerFull = {
	...serverFullResponse,
	id: auditLogEntryExamples.server_id,
	name: auditLogEntryExamples.server.name,
	subdomain: auditLogEntryExamples.server.subdomain,
	worlds: auditLogEntryExamples.worlds.map((world, index) => ({
		...serverFullResponse.worlds[Math.min(index, serverFullResponse.worlds.length - 1)],
		id: world.id,
		name: world.name,
		backups: index === 0 ? auditLogEntryExamples.backups : [],
	})),
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
			icon_url:
				'https://cdn.modrinth.com/data/oWaK0Q19/f66b5589924884ffd81acb27f3ccb775867a962e_96.webp',
			version: null,
		},
		AANobbMI: {
			title: 'Sodium',
			slug: 'sodium',
			icon_url:
				'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',
			version: null,
		},
		P7dR8mSH: {
			title: 'Fabric API',
			slug: 'fabric-api',
			icon_url: 'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
			version: null,
		},
		'project-modpack-001': {
			title: 'Vault Hunters',
			slug: 'vault-hunters',
			icon_url: null,
			version: null,
		},
	},
	versions: {
		HY8u0JqC: { name: 'Create Aeronautics 1.0.0', version_number: '1.0.0' },
		yaoBL9D9: { name: 'Sodium 0.6.0', version_number: '0.6.0' },
		KZS9tylY: { name: 'Fabric API 0.116.0', version_number: '0.116.0' },
		'version-modpack-001': { name: 'Vault Hunters 3.15.1', version_number: '3.15.1' },
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
				metadata: { user_id: userIds.modmuss, permissions: -4611686018427387904 },
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
					permissions: -288230376151711744,
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
			metadata: {
				spec: {
					platform: 'modrinth',
					project_id: 'project-modpack-001',
					version_id: 'version-modpack-001',
				},
			},
			minutesAgo: 80,
		}),
		rawEntry({
			action: 'modpack_unlinked',
			metadata: {
				spec: {
					platform: 'modrinth',
					project_id: 'project-modpack-001',
					version_id: 'version-modpack-001',
				},
			},
			minutesAgo: 85,
		}),
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
			metadata: { new_loader: 'fabric', new_version: '0.16.10' },
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
	versions: {},
	data: [
		rawEntry({
			action: 'user_permission_modified',
			metadata: { user_id: 'unknown-user', permissions: -9223372036854775808 },
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
			metadata: {
				spec: {
					platform: 'local_file',
					filename: 'example-modpack-v1.20.mrpack',
				},
			},
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

const overflowAddonNames = [
	"Alpha's Rise",
	'Bookshelf',
	'Diagonal Fences',
	'Dragon Compass',
	'Dragon Feed',
	'Dragons of the Cosmos',
	"Farmer's Delight",
	'Fastload',
	'Friendly Fire',
	'Isle of Berk Addons',
	'Jade',
	'Kiwi',
	"Nature's Compass",
	'Off the Grid Dragons',
	'Patchouli',
	'Puzzles Lib',
	'Resourceful Config',
	'Waystones',
]

const overflowActionLogResponse: Archon.Actions.v1.ActionLogResponse = {
	next_offset: null,
	users: actionLogResponse.users,
	addons: {
		...Object.fromEntries(
			overflowAddonNames.map((title, index) => [
				`overflow-addon-${index}`,
				{
					title,
					slug: `overflow-addon-${index}`,
					icon_url: null,
					version: null,
				},
			]),
		),
		'overflow-sodium': {
			title: 'Sodium',
			slug: 'sodium',
			icon_url: null,
			version: null,
		},
		'overflow-iris': {
			title: 'Iris Shaders',
			slug: 'iris',
			icon_url: null,
			version: null,
		},
	} as Record<string, Archon.Actions.v1.AddonResp>,
	versions: {
		...Object.fromEntries(
			overflowAddonNames.map((title, index) => [
				`overflow-version-${index}`,
				{
					name: `${title} 1.${index}.0`,
					version_number: `1.${index}.0`,
				},
			]),
		),
		'overflow-sodium-version': {
			name: 'Sodium mc1.21.1-0.6.13-fabric',
			version_number: 'mc1.21.1-0.6.13-fabric',
		},
		'overflow-iris-version': {
			name: 'Iris Shaders 1.8.8+1.21.1-fabric',
			version_number: '1.8.8+1.21.1-fabric',
		},
	} as Record<string, Archon.Actions.v1.VersionResp>,
	data: [
		rawEntry({
			action: 'addon_uploaded',
			metadata: {
				file_names: ['fabric-api-0.116.12+1.21.1.jar', 'lithium-fabric-0.15.3+mc1.21.1.jar'],
			},
			minutesAgo: 1,
		}),
		rawEntry({
			action: 'addon_added',
			metadata: {
				addons: [
					{ addon_id: 'overflow-sodium', version_id: 'overflow-sodium-version' },
					{ addon_id: 'overflow-iris', version_id: 'overflow-iris-version' },
				],
			},
			minutesAgo: 2,
		}),
		rawEntry({
			action: 'addon_enabled',
			metadata: {
				addons: [
					{ addon_id: 'overflow-sodium', version_id: 'overflow-sodium-version' },
					{ addon_id: 'overflow-iris', version_id: 'overflow-iris-version' },
				],
			},
			minutesAgo: 3,
		}),
		rawEntry({
			action: 'addon_disabled',
			metadata: {
				addons: [
					{ addon_id: 'overflow-sodium', version_id: 'overflow-sodium-version' },
					{ addon_id: 'overflow-iris', version_id: 'overflow-iris-version' },
				],
			},
			minutesAgo: 4,
		}),
		rawEntry({
			action: 'addon_deleted',
			metadata: {
				addons: overflowAddonNames.map((_, index) => ({
					addon_id: `overflow-addon-${index}`,
					version_id: `overflow-version-${index}`,
				})),
			},
			minutesAgo: 5,
		}),
		rawEntry({
			action: 'startup_command_modified',
			metadata: {
				command:
					'java -Xmx8192M -XX:+UseG1GC -XX:+UnlockExperimentalVMOptions -jar fabric-server-launch.jar nogui',
			},
			minutesAgo: 6,
		}),
		rawEntry({
			action: 'server_properties_modified',
			metadata: {
				properties: {
					difficulty: 'hard',
					'enforce-whitelist': 'true',
					'max-players': '20',
					'resource-pack': 'https://cdn.example.invalid/very-long-resource-pack-name.zip',
				},
			},
			minutesAgo: 7,
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
		versions: response.versions,
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

const everyActionEntries = toAuditEntries(
	auditLogEntryExamples,
	auditLogEntryExamplesServerFullResponse,
)
const sampleEntries = toAuditEntries(actionLogResponse)
const permissionBadgeEntries = sampleEntries.filter((entry) =>
	['user_invited', 'user_permission_modified'].includes(entry.event.key),
)
const loaderVersionEntries = toAuditEntries({
	...actionLogResponse,
	data: [
		rawEntry({
			action: 'loader_version_edited',
			metadata: { new_loader: 'fabric', new_version: '0.19.2' },
			minutesAgo: 1,
		}),
		rawEntry({
			action: 'loader_version_edited',
			metadata: { new_loader: null, new_version: '0.30.0-beta.7' },
			minutesAgo: 2,
		}),
		rawEntry({
			action: 'loader_version_edited',
			metadata: { new_loader: null, new_version: null },
			minutesAgo: 3,
		}),
	],
})
const fallbackEntries = toAuditEntries(missingLookupActionLogResponse, {
	...serverFullResponse,
	worlds: [],
})
const overflowEntries = toAuditEntries(overflowActionLogResponse)

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

function createTimeframeState(
	initial: Partial<{
		mode: TimeFrameMode
		preset: TimeFramePreset
		lastAmount: number
		lastUnit: TimeFrameLastUnit
		customStartDate: string
		customEndDate: string
	}> = {},
) {
	return {
		timeframeCustomEndDate: ref(initial.customEndDate ?? ''),
		timeframeCustomStartDate: ref(initial.customStartDate ?? ''),
		timeframeLastAmount: ref(initial.lastAmount ?? 30),
		timeframeLastUnit: ref<TimeFrameLastUnit>(initial.lastUnit ?? 'days'),
		timeframeMode: ref<TimeFrameMode>(initial.mode ?? 'preset'),
		timeframePreset: ref<TimeFramePreset>(initial.preset ?? 'all_time'),
	}
}

function renderStory(entries: ServerAuditLogEntry[], initialQuery = '') {
	return () => ({
		components: { AuditLogTable },
		setup() {
			const query = ref(initialQuery)
			const filters = ref<ServerAuditLogFilters>({
				userId: null,
				worldId: null,
			})
			return { entries, filters, query, ...createTimeframeState() }
		},
		template: /* html */ `
			<AuditLogTable
				v-model:query="query"
				v-model:timeframe-mode="timeframeMode"
				v-model:timeframe-preset="timeframePreset"
				v-model:timeframe-last-amount="timeframeLastAmount"
				v-model:timeframe-last-unit="timeframeLastUnit"
				v-model:timeframe-custom-start-date="timeframeCustomStartDate"
				v-model:timeframe-custom-end-date="timeframeCustomEndDate"
				v-model:filters="filters"
				:entries="entries"
			/>
		`,
	})
}

export const AllEvents: Story = {
	render: renderStory(everyActionEntries),
}

export const PermissionBadges: Story = {
	render: renderStory(permissionBadgeEntries),
}

export const LoaderVersionEvents: Story = {
	render: renderStory(loaderVersionEntries),
}

export const LongOverflowTooltip: Story = {
	render: renderStory(overflowEntries),
}

export const MissingLookupsAndFallbacks: Story = {
	render: renderStory(fallbackEntries),
}

export const Filtered: Story = {
	render: renderStory(sampleEntries, 'Create Aeronautics'),
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
				entries: sampleEntries,
				externalFilters,
				filters,
				query,
				...createTimeframeState(),
			}
		},
		template: /* html */ `
			<AuditLogTable
				v-model:query="query"
				v-model:timeframe-mode="timeframeMode"
				v-model:timeframe-preset="timeframePreset"
				v-model:timeframe-last-amount="timeframeLastAmount"
				v-model:timeframe-last-unit="timeframeLastUnit"
				v-model:timeframe-custom-start-date="timeframeCustomStartDate"
				v-model:timeframe-custom-end-date="timeframeCustomEndDate"
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
			return { entries: [], filters, query, ...createTimeframeState() }
		},
		template: /* html */ `
			<AuditLogTable
				v-model:query="query"
				v-model:timeframe-mode="timeframeMode"
				v-model:timeframe-preset="timeframePreset"
				v-model:timeframe-last-amount="timeframeLastAmount"
				v-model:timeframe-last-unit="timeframeLastUnit"
				v-model:timeframe-custom-start-date="timeframeCustomStartDate"
				v-model:timeframe-custom-end-date="timeframeCustomEndDate"
				v-model:filters="filters"
				:entries="entries"
				has-active-external-filters
			/>
		`,
	}),
}

export const MediumAppWindow: Story = {
	render: () => ({
		components: { AuditLogTable },
		setup() {
			const query = ref('')
			const filters = ref<ServerAuditLogFilters>({
				userId: null,
				worldId: null,
			})
			return {
				entries: everyActionEntries.slice(0, 8),
				filters,
				query,
				...createTimeframeState(),
			}
		},
		template: /* html */ `
			<div style="max-width: 860px;">
				<AuditLogTable
					v-model:query="query"
					v-model:timeframe-mode="timeframeMode"
					v-model:timeframe-preset="timeframePreset"
					v-model:timeframe-last-amount="timeframeLastAmount"
					v-model:timeframe-last-unit="timeframeLastUnit"
					v-model:timeframe-custom-start-date="timeframeCustomStartDate"
					v-model:timeframe-custom-end-date="timeframeCustomEndDate"
					v-model:filters="filters"
					:entries="entries"
				/>
			</div>
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
			return {
				entries: everyActionEntries.slice(0, 8),
				filters,
				query,
				...createTimeframeState(),
			}
		},
		template: /* html */ `
			<div style="max-width: 390px;">
				<AuditLogTable
					v-model:query="query"
					v-model:timeframe-mode="timeframeMode"
					v-model:timeframe-preset="timeframePreset"
					v-model:timeframe-last-amount="timeframeLastAmount"
					v-model:timeframe-last-unit="timeframeLastUnit"
					v-model:timeframe-custom-start-date="timeframeCustomStartDate"
					v-model:timeframe-custom-end-date="timeframeCustomEndDate"
					v-model:filters="filters"
					:entries="entries"
				/>
			</div>
		`,
	}),
}
