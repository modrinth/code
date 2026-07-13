import type { Meta, StoryObj } from '@storybook/vue3-vite'

import type { SharedPlayersTableRow } from '../../components/sharing/shared-players-table-types'
import SharedPlayersTable from '../../components/sharing/SharedPlayersTable.vue'

const now = Date.now()
const daysAgo = (days: number) => new Date(now - days * 24 * 60 * 60 * 1000)

const rows: SharedPlayersTableRow[] = [
	{
		id: 'coolbot',
		username: 'Coolbot',
		lastPlayedAt: null,
		joinedAt: null,
		method: 'direct',
		pending: true,
	},
	{
		id: 'geometrically',
		username: 'Geometrically',
		lastPlayedAt: daysAgo(0),
		joinedAt: daysAgo(0),
		method: 'link',
	},
	{
		id: 'boris',
		username: 'Boris',
		lastPlayedAt: daysAgo(4),
		joinedAt: daysAgo(7),
		method: 'direct',
	},
]

const meta = {
	title: 'Sharing/SharedPlayersTable',
	component: SharedPlayersTable,
	parameters: {
		layout: 'padded',
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div style="max-width: 1100px;"><story /></div>',
		}),
	],
	args: {
		rows,
	},
} satisfies Meta<typeof SharedPlayersTable>

export default meta
type Story = StoryObj<typeof meta>

export const InstanceShare: Story = {}

export const ServerPlay: Story = {
	args: {
		variant: 'server',
	},
}
