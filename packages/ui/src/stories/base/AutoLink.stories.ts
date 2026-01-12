import type { Meta, StoryObj } from '@storybook/vue3-vite'

import AutoLink from '../../components/base/AutoLink.vue'

const meta = {
	title: 'Base/AutoLink',
	component: AutoLink,
	render: (args) => ({
		components: { AutoLink },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<AutoLink v-bind="args">Link Text</AutoLink>
		`,
	}),
} satisfies Meta<typeof AutoLink>

export default meta
type Story = StoryObj<typeof meta>

export const ExternalLink: Story = {
	args: {
		to: 'https://modrinth.com',
	},
}

export const InternalPath: Story = {
	args: {
		to: '/projects',
	},
}
