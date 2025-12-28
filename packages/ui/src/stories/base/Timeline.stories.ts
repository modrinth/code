import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Timeline from '../../components/base/Timeline.vue'

const meta = {
	title: 'Base/Timeline',
	component: Timeline,
	render: (args) => ({
		components: { Timeline },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<Timeline v-bind="args">
				<div style="display: flex; gap: 0.5rem; align-items: center;">
					<div style="width: 1rem; height: 1rem; border-radius: 50%; background: var(--color-brand);"></div>
					<span>Event 1</span>
				</div>
				<div style="display: flex; gap: 0.5rem; align-items: center;">
					<div style="width: 1rem; height: 1rem; border-radius: 50%; background: var(--color-brand);"></div>
					<span>Event 2</span>
				</div>
				<div style="display: flex; gap: 0.5rem; align-items: center;">
					<div style="width: 1rem; height: 1rem; border-radius: 50%; background: var(--color-brand);"></div>
					<span>Event 3</span>
				</div>
			</Timeline>
		`,
	}),
} satisfies Meta<typeof Timeline>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const FadeOutStart: Story = {
	args: {
		fadeOutStart: true,
	},
}

export const FadeOutEnd: Story = {
	args: {
		fadeOutEnd: true,
	},
}
