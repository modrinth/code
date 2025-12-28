import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Avatar from '../../components/base/Avatar.vue'

const meta = {
	title: 'Base/Avatar',
	component: Avatar,
} satisfies Meta<typeof Avatar>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithImage: Story = {
	args: {
		src: 'https://cdn.modrinth.com/data/AANobbMI/icon.png',
		alt: 'Project icon',
	},
}

export const Circle: Story = {
	args: {
		src: 'https://cdn.modrinth.com/data/AANobbMI/icon.png',
		circle: true,
	},
}

export const AllSizes: Story = {
	render: () => ({
		components: { Avatar },
		template: /*html*/ `
			<div style="display: flex; gap: 1rem; align-items: center;">
				<Avatar src="https://cdn.modrinth.com/data/AANobbMI/icon.png" size="1.5rem" />
				<Avatar src="https://cdn.modrinth.com/data/AANobbMI/icon.png" size="2rem" />
				<Avatar src="https://cdn.modrinth.com/data/AANobbMI/icon.png" size="3rem" />
				<Avatar src="https://cdn.modrinth.com/data/AANobbMI/icon.png" size="4rem" />
			</div>
		`,
	}),
}

export const Placeholder: Story = {
	args: {
		size: '3rem',
	},
}
