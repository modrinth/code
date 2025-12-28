import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Toggle from '../../components/base/Toggle.vue'

const meta = {
	title: 'Base/Toggle',
	component: Toggle,
} satisfies Meta<typeof Toggle>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		modelValue: false,
	},
}

export const Checked: Story = {
	args: {
		modelValue: true,
	},
}

export const Disabled: Story = {
	args: {
		modelValue: false,
		disabled: true,
	},
}

export const AllStates: Story = {
	render: () => ({
		components: { Toggle },
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem;">
				<div style="display: flex; align-items: center; gap: 0.5rem;">
					<Toggle :model-value="false" /> Off
				</div>
				<div style="display: flex; align-items: center; gap: 0.5rem;">
					<Toggle :model-value="true" /> On
				</div>
				<div style="display: flex; align-items: center; gap: 0.5rem;">
					<Toggle :model-value="false" :disabled="true" /> Disabled
				</div>
			</div>
		`,
	}),
}
