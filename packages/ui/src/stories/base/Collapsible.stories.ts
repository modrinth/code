import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Collapsible from '../../components/base/Collapsible.vue'

const meta = {
	title: 'Base/Collapsible',
	component: Collapsible,
	render: (args) => ({
		components: { Collapsible },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<Collapsible v-bind="args">
				<p>This content can be collapsed or expanded.</p>
			</Collapsible>
		`,
	}),
} satisfies Meta<typeof Collapsible>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		collapsed: false,
	},
}

export const Collapsed: Story = {
	args: {
		collapsed: true,
	},
}
