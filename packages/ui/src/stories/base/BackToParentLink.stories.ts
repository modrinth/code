import type { Meta, StoryObj } from '@storybook/vue3-vite'

import BackToParentLink from '../../components/base/BackToParentLink.vue'

const meta = {
	title: 'Base/BackToParentLink',
	component: BackToParentLink,
	render: (args) => ({
		components: { BackToParentLink },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<BackToParentLink v-bind="args">All versions</BackToParentLink>
		`,
	}),
} satisfies Meta<typeof BackToParentLink>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		to: '/versions',
	},
}
