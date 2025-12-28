import type { Meta, StoryObj } from '@storybook/vue3-vite'

import FileInput from '../../components/base/FileInput.vue'

const meta = {
	title: 'Base/FileInput',
	component: FileInput,
} satisfies Meta<typeof FileInput>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		prompt: 'Select file',
	},
}

export const Multiple: Story = {
	args: {
		prompt: 'Select files',
		multiple: true,
	},
}

export const ImagesOnly: Story = {
	args: {
		prompt: 'Select image',
		accept: 'image/*',
	},
}

export const Disabled: Story = {
	args: {
		prompt: 'Select file',
		disabled: true,
	},
}
