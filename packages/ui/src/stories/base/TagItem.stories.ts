import type { Meta, StoryObj } from '@storybook/vue3-vite'

import TagItem from '../../components/base/TagItem.vue'

const meta = {
	title: 'Base/TagItem',
	component: TagItem,
	render: (args) => ({
		components: { TagItem },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<TagItem v-bind="args">Tag Name</TagItem>
		`,
	}),
} satisfies Meta<typeof TagItem>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithAction: Story = {
	args: {
		action: () => alert('Tag clicked!'),
	},
}

export const MultipleTags: Story = {
	render: () => ({
		components: { TagItem },
		template: /*html*/ `
			<div style="display: flex; flex-wrap: wrap; gap: 0.5rem;">
				<TagItem>Minecraft</TagItem>
				<TagItem>Fabric</TagItem>
				<TagItem>Adventure</TagItem>
				<TagItem>Technology</TagItem>
			</div>
		`,
	}),
}
