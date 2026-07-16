import { UploadIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import FileButton from '../../components/base/buttons/FileButton.vue'

const meta = {
	title: 'Buttons/File Button',
	component: FileButton,
	argTypes: {
		variant: {
			control: 'select',
			options: ['base', 'colored', 'outlined', 'quiet'],
		},
		size: {
			control: 'select',
			options: ['sm', 'default', 'md', 'lg'],
		},
		tone: {
			control: 'select',
			options: ['brand', 'red', 'orange', 'green', 'blue', 'purple', 'promotion'],
		},
	},
	args: {
		prompt: 'Select file',
		variant: 'base',
		size: 'default',
		multiple: false,
		disabled: false,
	},
	render: (args) => ({
		components: { FileButton, UploadIcon },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<FileButton v-bind="args">
				<UploadIcon />
			</FileButton>
		`,
	}),
} satisfies Meta<typeof FileButton>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const MultipleImages: Story = {
	args: {
		prompt: 'Select images',
		accept: 'image/*',
		multiple: true,
		variant: 'colored',
	},
}

export const Disabled: Story = {
	args: {
		disabled: true,
	},
}
