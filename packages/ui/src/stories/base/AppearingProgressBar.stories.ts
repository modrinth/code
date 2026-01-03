import type { Meta, StoryObj } from '@storybook/vue3-vite'

import AppearingProgressBar from '../../components/base/AppearingProgressBar.vue'

const meta = {
	title: 'Base/AppearingProgressBar',
	component: AppearingProgressBar,
} satisfies Meta<typeof AppearingProgressBar>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		maxValue: 100,
		currentValue: 45,
	},
}

export const AllProgress: Story = {
	...Default,
	render: () => ({
		components: { AppearingProgressBar },
		template: `
			<div class="flex flex-col gap-8">
				<div>
					<p class="text-secondary mb-2">0%</p>
					<AppearingProgressBar :maxValue="100" :currentValue="0" />
				</div>
				<div>
					<p class="text-secondary mb-2">25%</p>
					<AppearingProgressBar :maxValue="100" :currentValue="25" />
				</div>
				<div>
					<p class="text-secondary mb-2">50%</p>
					<AppearingProgressBar :maxValue="100" :currentValue="50" />
				</div>
				<div>
					<p class="text-secondary mb-2">75%</p>
					<AppearingProgressBar :maxValue="100" :currentValue="75" />
				</div>
				<div>
					<p class="text-secondary mb-2">100%</p>
					<AppearingProgressBar :maxValue="100" :currentValue="100" />
				</div>
			</div>
		`,
	}),
}

export const CustomTips: Story = {
	args: {
		maxValue: 1000000,
		currentValue: 450000,
		tips: ['Loading assets...', 'Processing data...', 'Almost there...'],
	},
}
