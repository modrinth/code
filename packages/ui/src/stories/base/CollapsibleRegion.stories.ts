import type { Meta, StoryObj } from '@storybook/vue3-vite'

import CollapsibleRegion from '../../components/base/CollapsibleRegion.vue'

const meta = {
	title: 'Base/CollapsibleRegion',
	component: CollapsibleRegion,
} satisfies Meta<typeof CollapsibleRegion>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { CollapsibleRegion },
		template: `
			<CollapsibleRegion>
				<div class="space-y-4">
					<p>This is some content that can be collapsed or expanded.</p>
					<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</p>
					<p>Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>
					<p>Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.</p>
					<p>Duis aute irure dolor in reprehenderit in voluptate velit esse cillum.</p>
					<p>Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia.</p>
				</div>
			</CollapsibleRegion>
		`,
	}),
}

export const CustomLabels: Story = {
	render: () => ({
		components: { CollapsibleRegion },
		template: `
			<CollapsibleRegion expandText="Show more" collapseText="Show less" collapsedHeight="6rem">
				<div class="space-y-4">
					<p>Custom expand and collapse labels.</p>
					<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</p>
					<p>Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>
					<p>Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.</p>
					<p>Duis aute irure dolor in reprehenderit in voluptate velit esse cillum.</p>
				</div>
			</CollapsibleRegion>
		`,
	}),
}
