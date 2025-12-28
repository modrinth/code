import type { Meta, StoryObj } from '@storybook/vue3-vite'

import DropzoneFileInput from '../../components/base/DropzoneFileInput.vue'

const meta = {
	title: 'Base/DropzoneFileInput',
	component: DropzoneFileInput,
} satisfies Meta<typeof DropzoneFileInput>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Small: Story = {
	args: {
		size: 'small',
	},
}

export const MultipleFiles: Story = {
	args: {
		multiple: true,
		primaryPrompt: 'Drag and drop multiple files',
		secondaryPrompt: 'Select multiple files at once',
	},
}

export const Disabled: Story = {
	args: {
		disabled: true,
	},
}

export const CustomPrompts: Story = {
	args: {
		primaryPrompt: 'Drop your mod files here',
		secondaryPrompt: 'Supports .jar and .zip files up to 25MB',
		accept: '.jar,.zip',
	},
}
