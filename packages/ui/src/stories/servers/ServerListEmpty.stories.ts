import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ServerListEmpty from '../../components/servers/server-list-empty/ServerListEmpty.vue'

const meta = {
	title: 'Servers/ServerListEmpty',
	component: ServerListEmpty,
	parameters: {
		layout: 'centered',
	},
} satisfies Meta<typeof ServerListEmpty>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithImages: Story = {
	args: {
		gridImages: [
			'https://cdn.modrinth.com/data/AANobbMI/icon.png',
			'https://cdn.modrinth.com/data/P7dR8mSH/icon.png',
			'https://cdn.modrinth.com/data/gvQqBUqZ/icon.png',
			'https://cdn.modrinth.com/data/YL57xq9U/icon.png',
			'https://cdn.modrinth.com/data/mOgUt4GM/icon.png',
			'https://cdn.modrinth.com/data/9eGKb6K1/icon.png',
		],
	},
}
