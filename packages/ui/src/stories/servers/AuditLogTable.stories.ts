import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import AuditLogTable from '../../components/servers/access/AuditLogTable.vue'
import type {
	ServerAccessMember,
	ServerAuditLogEntry,
	ServerAuditLogFilters,
} from '../../components/servers/access/types'

const users: ServerAccessMember[] = [
	{
		id: 'owner',
		user: { id: 'prospector', username: 'Prospector' },
		role: 'owner',
		joinedAt: new Date().toISOString(),
		isOwner: true,
	},
	{
		id: 'editor',
		user: { id: 'geometrically', username: 'Geometrically' },
		role: 'editor',
		joinedAt: new Date().toISOString(),
	},
]

const worlds = [
	{ id: 'create-smp', name: 'Create SMP' },
	{ id: 'smp-season-4', name: 'SMP Season 4' },
]

const entries: ServerAuditLogEntry[] = [
	{
		id: 'support',
		actor: { id: 'support', username: 'Support' },
		world: null,
		action: { type: 'file_edited', file: 'server.properties' },
		timestamp: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
	},
	{
		id: 'world',
		actor: users[1].user,
		world: null,
		action: { type: 'world_started', worldName: 'Create SMP' },
		timestamp: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString(),
	},
	{
		id: 'mod',
		actor: users[1].user,
		world: worlds[1],
		action: { type: 'content_installed', contentType: 'mod', name: 'Create Aeronautics' },
		timestamp: new Date(Date.now() - 6 * 60 * 60 * 1000).toISOString(),
	},
]

const meta = {
	title: 'Servers/AuditLogTable',
	component: AuditLogTable,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 1040px;"><story /></div>',
		}),
	],
} satisfies Meta<typeof AuditLogTable>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { AuditLogTable },
		setup() {
			const query = ref('')
			const filters = ref<ServerAuditLogFilters>({
				userId: null,
				worldId: null,
				actionType: null,
			})
			return { entries, users, worlds, query, filters }
		},
		template: /* html */ `
			<AuditLogTable
				v-model:query="query"
				v-model:filters="filters"
				:entries="entries"
				:users="users"
				:worlds="worlds"
			/>
		`,
	}),
}

export const Filtered: Story = {
	render: () => ({
		components: { AuditLogTable },
		setup() {
			const query = ref('server.properties')
			const filters = ref<ServerAuditLogFilters>({
				userId: null,
				worldId: null,
				actionType: 'file_edited',
			})
			return { entries, users, worlds, query, filters }
		},
		template: /* html */ `
			<AuditLogTable
				v-model:query="query"
				v-model:filters="filters"
				:entries="entries"
				:users="users"
				:worlds="worlds"
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
				actionType: null,
			})
			return { entries, users, worlds, query, filters }
		},
		template: /* html */ `
			<div style="max-width: 390px;">
				<AuditLogTable
					v-model:query="query"
					v-model:filters="filters"
					:entries="entries"
					:users="users"
					:worlds="worlds"
				/>
			</div>
		`,
	}),
}
