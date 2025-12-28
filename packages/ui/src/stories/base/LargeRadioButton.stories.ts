import type { Meta, StoryObj } from '@storybook/vue3-vite'

import LargeRadioButton from '../../components/base/LargeRadioButton.vue'

const meta = {
	title: 'Base/LargeRadioButton',
	component: LargeRadioButton as any,
} satisfies Meta<any>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {},
	render: () => ({
		components: { LargeRadioButton },
		template: `
			<LargeRadioButton :selected="false">
				Unselected option
			</LargeRadioButton>
		`,
	}),
}

export const AllStates: Story = {
	args: {},
	render: () => ({
		components: { LargeRadioButton },
		template: `
			<div class="flex flex-col gap-4 max-w-md">
				<LargeRadioButton :selected="false">
					Unselected option
				</LargeRadioButton>
				<LargeRadioButton :selected="true">
					Selected option
				</LargeRadioButton>
				<LargeRadioButton :selected="false" :disabled="true">
					Disabled unselected
				</LargeRadioButton>
				<LargeRadioButton :selected="true" :disabled="true">
					Disabled selected
				</LargeRadioButton>
			</div>
		`,
	}),
}
