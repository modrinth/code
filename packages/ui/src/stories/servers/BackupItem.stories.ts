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

function makeBackup(overrides: Partial<Archon.Backups.v1.Backup> = {}): Archon.Backups.v1.Backup {
	return {
		id: 'backup-001',
		physical_id: 'phys-001',
		name: 'Backup #5',
		created_at: new Date(Date.now() - 1000 * 60 * 10).toISOString(),
		automated: false,
		status: 'done',
		interrupted: false,
		ongoing: false,
		locked: false,
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

export const Creating: Story = {
	name: 'Creating (in progress)',
	args: {
		backup: makeBackup({
			automated: true,
			name: 'Backup #3',
			status: 'in_progress',
			ongoing: true,
			task: {
				create: { progress: 0.4, state: 'ongoing' },
			},
		}),
	},
}

export const Restoring: Story = {
	name: 'Restoring (in progress)',
	args: {
		backup: makeBackup({
			name: 'Backup #5',
			automated: true,
			task: {
				restore: { progress: 0.6, state: 'ongoing' },
			},
		}),
	},
}

export const FailedCreate: Story = {
	name: 'Failed (create)',
	args: {
		backup: makeBackup({ status: 'error', name: 'Backup #4' }),
	},
}

export const FailedRestore: Story = {
	name: 'Failed (restore)',
	args: {
		backup: makeBackup({
			name: 'Backup #5',
			task: {
				restore: { progress: 0, state: 'failed' },
			},
		}),
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

export const AllStates: Story = {
	render: () => ({
		components: { BackupItem },
		setup() {
			const now = new Date(Date.now() - 1000 * 60 * 10).toISOString()

			function makeBackup(overrides: Partial<Archon.Backups.v1.Backup>): Archon.Backups.v1.Backup {
				return {
					id: 'backup-001',
					physical_id: 'phys-001',
					name: 'Backup #5',
					created_at: now,
					automated: false,
					status: 'done',
					interrupted: false,
					ongoing: false,
					locked: false,
					...overrides,
				}
			}

			return {
				manual: makeBackup({ name: 'Base finished!!' }),
				automated: makeBackup({ automated: true, name: 'Backup #2' }),
				creating: makeBackup({
					automated: true,
					name: 'Backup #3',
					status: 'in_progress',
					ongoing: true,
					task: { create: { progress: 0.4, state: 'ongoing' } },
				}),
				restoring: makeBackup({
					automated: true,
					task: { restore: { progress: 0.6, state: 'ongoing' } },
				}),
				failedCreate: makeBackup({ status: 'error', name: 'Backup #4' }),
				failedRestore: makeBackup({
					task: { restore: { progress: 0, state: 'failed' } },
				}),
			}
		},
		template: /* html */ `
			<div style="display: flex; flex-direction: column; gap: 0.75rem; max-width: 900px;">
				<BackupItem :backup="manual" />
				<BackupItem :backup="automated" />
				<BackupItem :backup="creating" />
				<BackupItem :backup="restoring" />
				<BackupItem :backup="failedCreate" />
				<BackupItem :backup="failedRestore" />
				<BackupItem :backup="manual" preview />
			</div>
		`,
	}),
}
