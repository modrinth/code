import type { Meta, StoryObj } from '@storybook/vue3-vite'

import UserAvatar from '../../components/user/UserAvatar.vue'

const src = 'https://cdn.modrinth.com/data/AANobbMI/icon.png'

const meta = {
	title: 'User/User Avatar',
	component: UserAvatar,
	args: {
		src,
		size: '32px',
	},
} satisfies Meta<typeof UserAvatar>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Online: Story = {
	args: {
		badge: true,
	},
}

export const Offline: Story = {
	args: {
		grayscale: true,
	},
}

export const OnSurface: Story = {
	render: (args) => ({
		components: { UserAvatar },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<div style="display: flex; gap: 1rem; padding: 1rem; background: var(--color-button-bg); border-radius: 1rem;">
				<UserAvatar v-bind="args" />
				<UserAvatar v-bind="args" badge />
				<UserAvatar v-bind="args" grayscale />
			</div>
		`,
	}),
}
