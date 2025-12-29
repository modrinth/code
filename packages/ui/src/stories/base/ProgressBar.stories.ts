import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProgressBar from '../../components/base/ProgressBar.vue'

const meta = {
	title: 'Base/ProgressBar',
	component: ProgressBar,
} satisfies Meta<typeof ProgressBar>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		progress: 0.5,
	},
}

export const WithLabel: Story = {
	args: {
		progress: 0.75,
		label: 'Uploading...',
		showProgress: true,
	},
}

export const AllColors: StoryObj = {
	render: () => ({
		components: { ProgressBar },
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem;">
				<ProgressBar :progress="0.6" color="brand" label="Brand" />
				<ProgressBar :progress="0.6" color="green" label="Green" />
				<ProgressBar :progress="0.6" color="red" label="Red" />
				<ProgressBar :progress="0.6" color="orange" label="Orange" />
				<ProgressBar :progress="0.6" color="blue" label="Blue" />
				<ProgressBar :progress="0.6" color="purple" label="Purple" />
				<ProgressBar :progress="0.6" color="gray" label="Gray" />
			</div>
		`,
	}),
}

export const Striped: Story = {
	args: {
		progress: 0.5,
		striped: true,
	},
}

export const FullWidth: Story = {
	args: {
		progress: 0.5,
		fullWidth: true,
	},
}
