import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Badge from '../../components/base/Badge.vue'

const meta = {
	title: 'Base/Badge',
	component: Badge,
} satisfies Meta<typeof Badge>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		type: 'approved',
	},
}

export const ProjectStatuses: StoryObj = {
	render: () => ({
		components: { Badge },
		template: /*html*/ `
			<div style="display: flex; flex-wrap: wrap; gap: 0.5rem;">
				<Badge type="approved" />
				<Badge type="unlisted" />
				<Badge type="private" />
				<Badge type="draft" />
				<Badge type="archived" />
				<Badge type="rejected" />
				<Badge type="processing" />
			</div>
		`,
	}),
}

export const UserRoles: StoryObj = {
	render: () => ({
		components: { Badge },
		template: /*html*/ `
			<div style="display: flex; flex-wrap: wrap; gap: 0.5rem;">
				<Badge type="admin" />
				<Badge type="moderator" />
				<Badge type="creator" />
			</div>
		`,
	}),
}

export const WithColor: Story = {
	args: {
		type: 'release',
		color: 'green',
	},
}
