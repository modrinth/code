import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, fn, userEvent } from 'storybook/test'

import GalleryGrid from '../../components/project/gallery/GalleryGrid.vue'

const imageUrl = new URL('../../../../blog/public/proud-of-you-2026/stage.webp', import.meta.url).href
const images = [
	{
		url: imageUrl,
		title: 'Parade stage',
		created: '2026-06-19T12:00:00Z',
		featured: false,
		ordering: 0,
	},
	{
		url: `${imageUrl}#second-image`,
		title: 'Stage seating',
		created: '2026-06-19T12:00:00Z',
		featured: false,
		ordering: 1,
	},
]

const meta = {
	title: 'Project/Gallery Grid',
	component: GalleryGrid,
	args: { images, onPreview: fn() },
	render: (args) => ({
		components: { GalleryGrid },
		setup: () => ({ args }),
		template: '<GalleryGrid v-bind="args" class="grid max-w-3xl grid-cols-2 gap-4" />',
	}),
} satisfies Meta<typeof GalleryGrid>

export default meta
type Story = StoryObj<typeof meta>

export const ReadOnly: Story = {
	play: async ({ args, canvas }) => {
		await expect(canvas.queryByRole('button', { name: /reorder image/i })).not.toBeInTheDocument()
		await userEvent.click(canvas.getByRole('button', { name: 'Stage seating' }))
		await expect(args.onPreview).toHaveBeenCalledWith(1)
	},
}
