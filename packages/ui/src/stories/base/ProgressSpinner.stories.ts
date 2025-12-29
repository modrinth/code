import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProgressSpinner from '../../components/base/ProgressSpinner.vue'

const meta = {
	title: 'Base/ProgressSpinner',
	component: ProgressSpinner,
} satisfies Meta<typeof ProgressSpinner>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		progress: 0.5,
	},
}

export const AllProgress: StoryObj = {
	render: () => ({
		components: { ProgressSpinner },
		template: /*html*/ `
			<div style="display: flex; gap: 1rem; align-items: center;">
				<ProgressSpinner :progress="0" />
				<ProgressSpinner :progress="0.25" />
				<ProgressSpinner :progress="0.5" />
				<ProgressSpinner :progress="0.75" />
				<ProgressSpinner :progress="1" />
			</div>
		`,
	}),
}
