import type { Archon } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import BackupItem from '../../components/servers/backups/BackupItem.vue'

const meta = {
	title: 'Servers/BackupItem',
	component: BackupItem,
} satisfies Meta<typeof BackupItem>

export default meta
type Story = StoryObj<typeof meta>

function makeBackup(
	overrides: Partial<Archon.BackupsQueue.v1.BackupQueueBackup> = {},
): Archon.BackupsQueue.v1.BackupQueueBackup {
	return {
		id: 'backup-001',
		name: 'Backup #5',
		created_at: new Date(Date.now() - 1000 * 60 * 10).toISOString(),
		automated: false,
		status: 'done',
		locked: false,
		history: [],
		...overrides,
	}
}

export const Manual: Story = {
	args: {
		backup: makeBackup({ name: 'Base finished!!' }),
	},
}

export const Automated: Story = {
	args: {
		backup: makeBackup({ automated: true, name: 'Automatic backup' }),
	},
}

export const LongName: Story = {
	args: {
		backup: makeBackup({
			automated: true,
			name: 'Before restoring "A very long backup name that should truncate nicely"',
		}),
	},
	decorators: [() => ({ template: '<div style="max-width: 420px;"><story /></div>' })],
}
