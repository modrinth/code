import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect } from 'storybook/test'

import OverlayScrollArea from '../../components/base/OverlayScrollArea.vue'

const meta = {
	title: 'Base/OverlayScrollArea',
	component: OverlayScrollArea,
	args: { autoHide: true },
	render: (args) => ({
		components: { OverlayScrollArea },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<div class="max-w-lg">
				<p>Scroll to the last item. Hover over the area to reveal an auto-hidden scrollbar.</p>
				<OverlayScrollArea v-bind="args" class="h-64 rounded-xl bg-surface-2">
					<div class="flex flex-col gap-4 p-4">
						<p v-for="i in 30" :key="i" class="m-0">Item {{ i }}</p>
					</div>
				</OverlayScrollArea>
			</div>
		`,
	}),
	play: async ({ canvasElement }) => {
		const viewport = canvasElement.querySelector<HTMLElement>('[data-overlayscrollbars-viewport]')!
		await expect(viewport.scrollHeight).toBeGreaterThan(viewport.clientHeight)
		viewport.scrollTop = viewport.scrollHeight
		await expect(viewport.scrollTop).toBeGreaterThan(0)
		await expect(viewport.scrollTop + viewport.clientHeight).toBe(viewport.scrollHeight)
		viewport.scrollTop = 0
	},
} satisfies Meta<typeof OverlayScrollArea>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const AlwaysVisible: Story = {
	args: { autoHide: false },
}
