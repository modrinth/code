import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import Textarea from '../../components/base/inputs/Textarea.vue'

const meta = {
	title: 'Inputs/Textarea',
	component: Textarea,
	argTypes: {
		resize: {
			control: 'select',
			options: ['none', 'vertical', 'both'],
		},
	},
} satisfies Meta<typeof Textarea>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		placeholder: 'Enter a description...',
	},
}

export const WithRows: Story = {
	args: {
		rows: 5,
		placeholder: 'Enter details...',
	},
}

export const Resizable: Story = {
	args: {
		resize: 'vertical',
		placeholder: 'Drag the bottom-right corner to resize...',
	},
}

export const AllStates: StoryObj = {
	render: () => ({
		components: { Textarea },
		setup() {
			const normalValue = ref('')
			const filledValue = ref('Some content that has been entered into the textarea.')
			const errorValue = ref('Invalid content')
			const readonlyValue = ref('This content is readonly')
			return { normalValue, filledValue, errorValue, readonlyValue }
		},
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem; max-width: 400px;">
				<Textarea v-model="normalValue" placeholder="Enter text..." />
				<Textarea v-model="filledValue" />
				<Textarea v-model="errorValue" error />
				<Textarea placeholder="Disabled..." disabled />
				<Textarea v-model="readonlyValue" readonly />
				<Textarea resize="vertical" placeholder="Drag to resize..." />
			</div>
		`,
	}),
}
