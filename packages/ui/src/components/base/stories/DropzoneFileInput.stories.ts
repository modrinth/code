import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'

import DropzoneFileInput from '../DropzoneFileInput.vue'

const meta = {
	title: 'Base/DropzoneFileInput',
	component: DropzoneFileInput,
	tags: ['autodocs'],
	argTypes: {
		size: {
			control: 'select',
			options: ['standard', 'small'],
		},
		multiple: { control: 'boolean' },
		disabled: { control: 'boolean' },
		shouldAlwaysReset: { control: 'boolean' },
		accept: { control: 'text' },
		maxSize: { control: 'number' },
		primaryPrompt: { control: 'text' },
		secondaryPrompt: { control: 'text' },
	},
	args: {
		size: 'standard',
		multiple: false,
		disabled: false,
		shouldAlwaysReset: false,
		primaryPrompt: 'Drag and drop files or click to browse',
		secondaryPrompt: 'You can try to drag files or folder or click this area to select it',
		onChange: fn(),
	},
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

export const ImagesOnly: Story = {
	args: {
		accept: 'image/*',
		primaryPrompt: 'Upload an image',
		secondaryPrompt: 'Accepts PNG, JPG, GIF, WebP, and other image formats',
	},
}

export const PDFOnly: Story = {
	args: {
		accept: '.pdf',
		primaryPrompt: 'Upload a PDF document',
		secondaryPrompt: 'Only PDF files are accepted',
	},
}

export const WithMaxSize: Story = {
	args: {
		maxSize: 5 * 1024 * 1024, // 5MB
		primaryPrompt: 'Upload a file (max 5MB)',
		secondaryPrompt: 'Files larger than 5MB will be rejected',
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

export const SmallImageUploader: Story = {
	args: {
		size: 'small',
		accept: 'image/*',
		primaryPrompt: 'Upload icon',
		secondaryPrompt: 'PNG or JPG, max 256x256',
	},
}

// Showcase both sizes
export const SizeComparison: Story = {
	render: () => ({
		components: { DropzoneFileInput },
		template: `
			<div style="display: flex; flex-direction: column; gap: 2rem;">
				<div>
					<h3 style="margin-bottom: 0.5rem; font-weight: 600;">Standard Size</h3>
					<DropzoneFileInput
						size="standard"
						primary-prompt="Standard dropzone"
						secondary-prompt="This is the default size for file uploads"
					/>
				</div>
				<div>
					<h3 style="margin-bottom: 0.5rem; font-weight: 600;">Small Size</h3>
					<DropzoneFileInput
						size="small"
						primary-prompt="Small dropzone"
						secondary-prompt="Compact size for smaller areas"
					/>
				</div>
			</div>
		`,
	}),
}
