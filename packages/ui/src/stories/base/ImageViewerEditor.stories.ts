import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { Button } from '../../components/base/buttons'
import ImageViewerEditor from '../../components/image-viewer-editor/index.vue'
import type { ImageViewerEditorItem } from '../../components/image-viewer-editor/types'
import { provideImageViewerEditor } from '../../providers'

const IMAGE_SVG = `
	<svg xmlns="http://www.w3.org/2000/svg" width="1600" height="900" viewBox="0 0 1600 900">
		<defs>
			<linearGradient id="sky" x1="0" y1="0" x2="1" y2="1">
				<stop stop-color="#172554" />
				<stop offset="1" stop-color="#14532d" />
			</linearGradient>
		</defs>
		<rect width="1600" height="900" fill="url(#sky)" />
		<circle cx="1260" cy="210" r="110" fill="#fde68a" opacity="0.9" />
		<path d="M0 720 360 390 680 690 1040 330 1600 760V900H0Z" fill="#0f172a" />
		<path d="M0 790 390 560 720 780 1110 510 1600 800V900H0Z" fill="#166534" />
	</svg>
`
const IMAGE_URL = `data:image/svg+xml,${encodeURIComponent(IMAGE_SVG)}`
const items: ImageViewerEditorItem[] = [
	{
		id: 'mountains',
		src: IMAGE_URL,
		alt: 'Stylised mountain landscape',
		title: 'Mountain base at sunset',
		description: 'Survival world · August 24 at 4:12 PM',
		editorSource: { id: 'mountains', path: 'mountains.svg' },
	},
	{
		id: 'valley',
		src: IMAGE_URL,
		alt: 'Stylised valley landscape',
		title: 'View from the valley',
		description: 'Survival world · August 24 at 4:18 PM',
		editorSource: { id: 'valley', path: 'valley.svg' },
	},
]

const meta = {
	title: 'Base/ImageViewerEditor',
	component: ImageViewerEditor,
} satisfies Meta<typeof ImageViewerEditor>

export default meta
type Story = StoryObj<typeof ImageViewerEditor>

function render(editor: 'enabled' | 'disabled') {
	return () => ({
		components: { Button, ImageViewerEditor },
		setup() {
			const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
			provideImageViewerEditor({
				loadEditorData: async () => ({
					source: new Blob([IMAGE_SVG], { type: 'image/svg+xml' }),
				}),
			})
			return {
				editor,
				items,
				open: () => viewer.value?.show(0),
				openEditor: () => viewer.value?.edit(0),
				viewer,
			}
		},
		template: /*html*/ `
			<div class="flex gap-2">
				<Button type="colored" color="brand" @click="open">Open image viewer</Button>
				<Button v-if="editor === 'enabled'" type="outlined" @click="openEditor">
					Open image editor
				</Button>
			</div>
			<ImageViewerEditor ref="viewer" :items="items" :editor="editor" />
		`,
	})
}

export const ViewerOnly: Story = {
	render: render('disabled'),
}

export const Editable: Story = {
	render: render('enabled'),
}
