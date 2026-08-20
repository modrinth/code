import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ServerListEmpty from '../../components/servers/server-list-empty/ServerListEmpty.vue'

const meta = {
	title: 'Servers/ServerListEmpty',
	component: ServerListEmpty,
	parameters: {
		layout: 'centered',
	},
	args: {
		onClickNewServer: () => undefined,
		onClickSignIn: () => undefined,
	},
} satisfies Meta<typeof ServerListEmpty>

export default meta
type Story = StoryObj<typeof meta>

export const SignedOut: Story = {
	args: {
		loggedIn: false,
	},
}

export const LoggedIn: Story = {
	args: {
		loggedIn: true,
	},
}

export const Narrow: Story = {
	args: {
		loggedIn: false,
	},
	parameters: {
		viewport: {
			defaultViewport: 'mobile2',
		},
	},
}
