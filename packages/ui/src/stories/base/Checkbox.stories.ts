import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Checkbox from '../../components/base/Checkbox.vue'

const meta = {
	title: 'Base/Checkbox',
	component: Checkbox,
} satisfies Meta<typeof Checkbox>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		label: 'Accept terms and conditions',
		modelValue: false,
	},
}

export const Checked: Story = {
	args: {
		label: 'Accept terms and conditions',
		modelValue: true,
	},
}

export const Disabled: Story = {
	args: {
		label: 'Disabled checkbox',
		modelValue: false,
		disabled: true,
	},
}

export const Indeterminate: Story = {
	args: {
		label: 'Indeterminate state',
		modelValue: false,
		indeterminate: true,
	},
}

export const AllStates: StoryObj = {
	render: () => ({
		components: { Checkbox },
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem;">
				<Checkbox label="Unchecked" :model-value="false" />
				<Checkbox label="Checked" :model-value="true" />
				<Checkbox label="Indeterminate" :model-value="false" :indeterminate="true" />
				<Checkbox label="Disabled unchecked" :model-value="false" :disabled="true" />
				<Checkbox label="Disabled checked" :model-value="true" :disabled="true" />
			</div>
		`,
	}),
}
