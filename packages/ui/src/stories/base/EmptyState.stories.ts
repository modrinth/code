import { BoxIcon, DatabaseBackupIcon, PlusIcon, ReceiptTextIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import { Button } from '../../components/base/buttons'
import EmptyState from '../../components/base/EmptyState.vue'

const meta = {
	title: 'Base/EmptyState',
	component: EmptyState,
	argTypes: {
		icon: { control: false },
	},
} satisfies Meta<typeof EmptyState>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		icon: BoxIcon,
		heading: 'No content installed',
		description: 'Browse or upload mods to get started',
	},
}

export const WithActions: StoryObj = {
	render: () => ({
		components: { EmptyState, Button, PlusIcon },
		setup: () => ({ DatabaseBackupIcon }),
		template: /*html*/ `
			<EmptyState
				:icon="DatabaseBackupIcon"
				heading="No backups yet"
				description="Create your first backup"
			>
				<template #actions>
					<Button type="colored" color="brand" size="lg"><PlusIcon />Create backup</Button>
				</template>
			</EmptyState>
		`,
	}),
}

export const WithoutDescription: Story = {
	args: {
		icon: ReceiptTextIcon,
		heading: 'No transactions',
	},
}

export const WithoutPattern: Story = {
	args: {
		icon: BoxIcon,
		heading: 'This folder is empty',
		description: 'There are no files or folders.',
		hidePattern: true,
	},
}
