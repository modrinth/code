import type { Meta, StoryObj } from '@storybook/vue3-vite'

import BackupProgressAdmonition from '../../components/servers/backups/BackupProgressAdmonition.vue'

const meta = {
	title: 'Servers/BackupProgressAdmonition',
	component: BackupProgressAdmonition,
	parameters: {
		layout: 'padded',
	},
} satisfies Meta<typeof BackupProgressAdmonition>

export default meta
type Story = StoryObj<typeof meta>

const justNow = new Date().toISOString()
const eightMinsAgo = new Date(Date.now() - 8 * 60 * 1000).toISOString()
const fiveHoursAgo = new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString()

export const AllStates: Story = {
	render: () => ({
		components: { BackupProgressAdmonition },
		setup() {
			const now = new Date().toISOString()
			const mins8 = new Date(Date.now() - 8 * 60 * 1000).toISOString()
			const hours5 = new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString()
			return { now, mins8, hours5 }
		},
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem; max-width: 1020px;">
				<h3 style="margin: 0; color: var(--color-contrast);">Backup Creation</h3>
				<BackupProgressAdmonition type="create" state="ongoing" :progress="0" backup-name="World Backup 1" :created-at="now" />
				<BackupProgressAdmonition type="create" state="ongoing" :progress="0.33" backup-name="World Backup 1" :created-at="mins8" />
				<BackupProgressAdmonition type="create" state="failed" :progress="0" backup-name="World Backup 1" :created-at="hours5" />

				<h3 style="margin: 1rem 0 0; color: var(--color-contrast);">Backup Restoration</h3>
				<BackupProgressAdmonition type="restore" state="ongoing" :progress="0" backup-name="World Backup 1" :created-at="now" />
				<BackupProgressAdmonition type="restore" state="ongoing" :progress="0.33" backup-name="World Backup 1" :created-at="mins8" />
				<BackupProgressAdmonition type="restore" state="done" :progress="1" backup-name="World Backup 1" :created-at="hours5" />
				<BackupProgressAdmonition type="restore" state="failed" :progress="0" backup-name="World Backup 1" :created-at="hours5" />
			</div>
		`,
	}),
}

export const BackupQueued: Story = {
	args: {
		type: 'create',
		state: 'ongoing',
		progress: 0,
		backupName: 'World Backup 1',
		createdAt: justNow,
	},
}

export const CreatingBackup: Story = {
	args: {
		type: 'create',
		state: 'ongoing',
		progress: 0.33,
		backupName: 'World Backup 1',
		createdAt: eightMinsAgo,
	},
}

export const BackupFailed: Story = {
	args: {
		type: 'create',
		state: 'failed',
		progress: 0,
		backupName: 'World Backup 1',
		createdAt: fiveHoursAgo,
	},
}

export const RestoreQueued: Story = {
	args: {
		type: 'restore',
		state: 'ongoing',
		progress: 0,
		backupName: 'World Backup 1',
		createdAt: justNow,
	},
}

export const RestoringBackup: Story = {
	args: {
		type: 'restore',
		state: 'ongoing',
		progress: 0.33,
		backupName: 'World Backup 1',
		createdAt: eightMinsAgo,
	},
}

export const RestoreSuccessful: Story = {
	args: {
		type: 'restore',
		state: 'done',
		progress: 1,
		backupName: 'World Backup 1',
		createdAt: fiveHoursAgo,
	},
}

export const RestoreFailed: Story = {
	args: {
		type: 'restore',
		state: 'failed',
		progress: 0,
		backupName: 'World Backup 1',
		createdAt: fiveHoursAgo,
	},
}
