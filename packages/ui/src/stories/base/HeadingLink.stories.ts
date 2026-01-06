import type { Meta, StoryObj } from '@storybook/vue3-vite'

import HeadingLink from '../../components/base/HeadingLink.vue'

const meta = {
	title: 'Base/HeadingLink',
	component: HeadingLink,
	render: (args) => ({
		components: { HeadingLink },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<HeadingLink v-bind="args">View All Projects</HeadingLink>
		`,
	}),
} satisfies Meta<typeof HeadingLink>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		to: '/projects',
	},
}
