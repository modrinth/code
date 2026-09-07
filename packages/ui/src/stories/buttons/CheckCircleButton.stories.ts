import type { Meta, StoryObj } from '@storybook/vue3-vite'

import CheckCircleButton from '../../components/base/buttons/CheckCircleButton.vue'

const meta = {
	title: 'Buttons/CheckCircleButton',
	component: CheckCircleButton,
	args: {
		checked: false,
		disabled: false,
	},
	render: (args) => ({
		components: { CheckCircleButton },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<div class="w-96">
				<CheckCircleButton v-bind="args">Fabric 26.2</CheckCircleButton>
			</div>
		`,
	}),
} satisfies Meta<typeof CheckCircleButton>

export default meta
type Story = StoryObj<typeof meta>

export const Playground: Story = {}

export const States: Story = {
	render: () => ({
		components: { CheckCircleButton },
		template: /*html*/ `
			<div class="flex w-96 flex-col gap-1" role="radiogroup" aria-label="Example choices">
				<CheckCircleButton :checked="true">Selected choice</CheckCircleButton>
				<CheckCircleButton :checked="false">Unselected choice</CheckCircleButton>
				<CheckCircleButton :checked="false" disabled>Disabled choice</CheckCircleButton>
			</div>
		`,
	}),
}
