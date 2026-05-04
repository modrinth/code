import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import AccessTable from '../../components/servers/access/AccessTable.vue'
import type {
	ServerAccessMember,
	ServerAccessRole,
	ServerAccessRoleOption,
} from '../../components/servers/access/types'

const roleOptions: ServerAccessRoleOption[] = [
	{
		value: 'owner',
		label: 'Owner',
		description: 'Full access including billing, members, and destructive actions.',
	},
	{
		value: 'editor',
		label: 'Editor',
		description: 'Manage server content, files, backups, and other settings.',
	},
	{
		value: 'viewer',
		label: 'Viewer',
		description: 'Start, stop, restart, and view the server without making changes.',
	},
]

const members: ServerAccessMember[] = [
	{
		id: 'owner',
		user: { id: 'prospector', username: 'Prospector' },
		role: 'owner',
		joinedAt: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
		isOwner: true,
	},
	{
		id: 'editor',
		user: { id: 'geometrically', username: 'Geometrically' },
		role: 'editor',
		joinedAt: new Date(Date.now() - 21 * 24 * 60 * 60 * 1000).toISOString(),
	},
	{
		id: 'pending',
		user: { id: 'imb', username: 'IMB' },
		role: 'viewer',
		joinedAt: null,
		pending: true,
	},
]

const meta = {
	title: 'Servers/AccessTable',
	component: AccessTable,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 960px;"><story /></div>',
		}),
	],
} satisfies Meta<typeof AccessTable>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { AccessTable },
		setup() {
			const rows = ref([...members])
			function updateRole(member: ServerAccessMember, role: ServerAccessRole) {
				rows.value = rows.value.map((row) => (row.id === member.id ? { ...row, role } : row))
			}
			function removeMember(member: ServerAccessMember) {
				rows.value = rows.value.filter((row) => row.id !== member.id)
			}
			return { rows, roleOptions, updateRole, removeMember }
		},
		template: /* html */ `
			<AccessTable
				:members="rows"
				:roles="roleOptions"
				@update-role="updateRole"
				@resend-invite="() => {}"
				@cancel-invite="removeMember"
				@remove-member="removeMember"
			/>
		`,
	}),
}

export const PendingInvite: Story = {
	args: {
		members: [members[2]],
		roles: roleOptions,
	},
}

export const OwnerFixed: Story = {
	args: {
		members: [members[0]],
		roles: roleOptions,
	},
}

export const MobileCompact: Story = {
	render: () => ({
		components: { AccessTable },
		setup() {
			const rows = ref([...members])
			function updateRole(member: ServerAccessMember, role: ServerAccessRole) {
				rows.value = rows.value.map((row) => (row.id === member.id ? { ...row, role } : row))
			}
			function removeMember(member: ServerAccessMember) {
				rows.value = rows.value.filter((row) => row.id !== member.id)
			}
			return { rows, roleOptions, updateRole, removeMember }
		},
		template: /* html */ `
			<div style="max-width: 390px;">
				<AccessTable
					:members="rows"
					:roles="roleOptions"
					@update-role="updateRole"
					@resend-invite="() => {}"
					@cancel-invite="removeMember"
					@remove-member="removeMember"
				/>
			</div>
		`,
	}),
}
