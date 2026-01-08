import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ScrollablePanel from '../../components/base/ScrollablePanel.vue'

const meta = {
	title: 'Base/ScrollablePanel',
	component: ScrollablePanel,
	render: (args) => ({
		components: { ScrollablePanel },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<ScrollablePanel v-bind="args">
				<div style="display: flex; flex-direction: column; gap: 1rem;">
					<p v-for="i in 20" :key="i">Item {{ i }}</p>
				</div>
			</ScrollablePanel>
		`,
	}),
} satisfies Meta<typeof ScrollablePanel>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const DisabledScrolling: Story = {
	args: {
		disableScrolling: true,
	},
}
