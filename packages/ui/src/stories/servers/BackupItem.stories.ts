import type { Archon } from '@modrinth/api-client'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import BackupItem from '../../components/servers/backups/BackupItem.vue'

const meta = {
	title: 'Servers/BackupItem',
	component: BackupItem,
	args: {
		preview: false,
		showCopyIdAction: false,
		showDebugInfo: false,
		restoreDisabled: undefined,
	},
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

export const Default: Story = {
	name: 'Default (manual)',
	args: {
		backup: makeBackup({ name: 'Base finished!!' }),
	},
}

export const Automated: Story = {
	name: 'Automated',
	args: {
		backup: makeBackup({ automated: true, name: 'Backup #2' }),
	},
}

export const Preview: Story = {
	name: 'Preview (compact, used in delete modal)',
	args: {
		backup: makeBackup({ name: 'Base finished!!' }),
		preview: true,
	},
}

export const RestoreDisabled: Story = {
	name: 'Restore disabled (server running)',
	args: {
		backup: makeBackup({ name: 'Backup #5', automated: true }),
		restoreDisabled: 'Cannot restore backup while server is running',
	},
}

export const CommonStates: Story = {
	render: () => ({
		components: { BackupItem },
		setup() {
			const now = new Date(Date.now() - 1000 * 60 * 10).toISOString()

			function makeBackup(
				overrides: Partial<Archon.BackupsQueue.v1.BackupQueueBackup>,
			): Archon.BackupsQueue.v1.BackupQueueBackup {
				return {
					id: 'backup-001',
					name: 'Backup #5',
					created_at: now,
					automated: false,
					status: 'done',
					locked: false,
					history: [],
					...overrides,
				}
			}

			return {
				manual: makeBackup({ name: 'Base finished!!' }),
				automated: makeBackup({ automated: true, name: 'Backup #2' }),
			}
		},
		template: /* html */ `
			<div style="display: flex; flex-direction: column; gap: 0.75rem; max-width: 900px;">
				<BackupItem :backup="manual" />
				<BackupItem :backup="automated" />
				<BackupItem :backup="manual" preview />
			</div>
		`,
	}),
}
