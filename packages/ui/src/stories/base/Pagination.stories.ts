import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Pagination from '../../components/base/Pagination.vue'

const meta = {
	title: 'Base/Pagination',
	component: Pagination,
} satisfies Meta<typeof Pagination>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		page: 1,
		count: 10,
	},
}

export const MiddlePage: Story = {
	args: {
		page: 5,
		count: 10,
	},
}

export const LastPage: Story = {
	args: {
		page: 10,
		count: 10,
	},
}

export const FewPages: Story = {
	args: {
		page: 1,
		count: 3,
	},
}

export const ManyPages: Story = {
	args: {
		page: 50,
		count: 100,
	},
}
