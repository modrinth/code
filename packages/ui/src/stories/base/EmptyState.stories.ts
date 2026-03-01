import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import EmptyState from '../../components/base/EmptyState.vue'

const meta = {
	title: 'Base/EmptyState',
	component: EmptyState,
	argTypes: {
		type: {
			control: 'select',
			options: [
				undefined,
				'done',
				'empty',
				'empty-inbox',
				'error',
				'no-connection',
				'no-credit-card',
				'no-documents',
				'no-gps',
				'no-images',
				'no-items-cart',
				'no-messages',
				'no-search-result',
				'no-tasks',
			],
		},
	},
} satisfies Meta<typeof EmptyState>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		type: 'empty-inbox',
		heading: 'No content installed',
		description: 'Browse or upload mods to get started',
	},
}

export const WithActions: StoryObj = {
	render: () => ({
		components: { EmptyState, ButtonStyled },
		template: /*html*/ `
			<EmptyState
				type="empty-inbox"
				heading="No backups yet"
				description="Create your first backup"
			>
				<template #actions>
					<ButtonStyled color="brand">
						<button>Create backup</button>
					</ButtonStyled>
				</template>
			</EmptyState>
		`,
	}),
}

export const TextOnly: Story = {
	args: {
		heading: 'No transactions',
		description: 'Your transaction history will appear here.',
	},
}

export const AllIllustrations: StoryObj = {
	render: () => ({
		components: { EmptyState },
		template: /*html*/ `
			<div class="grid grid-cols-2 gap-8">
				<EmptyState type="done" heading="Done" />
				<EmptyState type="empty" heading="Empty" />
				<EmptyState type="empty-inbox" heading="Empty Inbox" />
				<EmptyState type="error" heading="Error" />
				<EmptyState type="no-connection" heading="No Connection" />
				<EmptyState type="no-credit-card" heading="No Credit Card" />
				<EmptyState type="no-documents" heading="No Documents" />
				<EmptyState type="no-gps" heading="No GPS" />
				<EmptyState type="no-images" heading="No Images" />
				<EmptyState type="no-items-cart" heading="No Items in Cart" />
				<EmptyState type="no-messages" heading="No Messages" />
				<EmptyState type="no-search-result" heading="No Search Results" />
				<EmptyState type="no-tasks" heading="No Tasks" />
			</div>
		`,
	}),
}
