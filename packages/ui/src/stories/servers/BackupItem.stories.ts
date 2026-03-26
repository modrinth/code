import type { Archon } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import BackupItem from '../../components/servers/backups/BackupItem.vue'

const backup: Archon.Backups.v1.Backup = {
	id: 'backup_01JQ8J6A2P0V7K1M9R3S5T7U9W',
	physical_id: 'physical_01JQ8J6A2P0V7K1M9R3S5T7U9W',
	name: 'Pre-update backup',
	automated: false,
	created_at: '2026-03-24T14:32:00.000Z',
	status: 'done',
	interrupted: false,
	ongoing: false,
	locked: false,
}

const meta = {
	title: 'Servers/BackupItem',
	component: BackupItem,
	parameters: {
		layout: 'padded',
	},
} satisfies Meta<typeof BackupItem>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		backup,
		kyrosUrl: 'example.modrinth.gg',
		jwt: 'token',
	},
}

export const DeveloperMode: Story = {
	args: {
		backup,
		kyrosUrl: 'example.modrinth.gg',
		jwt: 'token',
		showCopyIdAction: true,
	},
}
