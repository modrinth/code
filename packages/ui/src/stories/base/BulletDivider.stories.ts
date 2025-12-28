import type { Meta, StoryObj } from '@storybook/vue3-vite'

import BulletDivider from '../../components/base/BulletDivider.vue'

const meta = {
	title: 'Base/BulletDivider',
	component: BulletDivider,
} satisfies Meta<typeof BulletDivider>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const InContext: Story = {
	render: () => ({
		components: { BulletDivider },
		template: /*html*/ `
			<div style="display: flex; align-items: center;">
				<span>Item 1</span>
				<BulletDivider />
				<span>Item 2</span>
				<BulletDivider />
				<span>Item 3</span>
			</div>
		`,
	}),
}
