import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import GalleryImageFields from '../../components/project/gallery/GalleryImageFields.vue'

const previewSrc = new URL('../../../../blog/public/proud-of-you-2026/stage.webp', import.meta.url)

const meta = {
	title: 'Project/Gallery Image Fields',
	component: GalleryImageFields,
	args: {
		accept: 'image/png,image/jpeg,image/gif,image/webp',
		previewSrc: previewSrc.href,
		title: 'Pride on the Modrinth SMP',
		description: 'A view of the stage before the parade.',
		featured: false,
	},
	render: (args) => ({
		components: { GalleryImageFields },
		setup: () => ({
			args,
			title: ref(args.title),
			description: ref(args.description),
			featured: ref(args.featured),
		}),
		template: `
			<div class="max-w-lg">
				<GalleryImageFields
					v-bind="args"
					v-model:title="title"
					v-model:description="description"
					v-model:featured="featured"
				/>
			</div>
		`,
	}),
} satisfies Meta<typeof GalleryImageFields>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
export const Upload: Story = { args: { filename: 'stage.webp', title: '', description: '' } }
