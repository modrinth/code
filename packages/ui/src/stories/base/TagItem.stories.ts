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

export const ColorVariants: Story = {
	render: () => ({
		components: { TagItem },
		template: /*html*/ `
			<div style="display: flex; flex-wrap: wrap; gap: 0.5rem;">
				<TagItem 	class="border !border-solid border-brand !font-medium" style="--_bg-color: var(--color-brand-highlight); --_color: var(--color-text-primary);">Brand</TagItem>
				<TagItem 	class="border !border-solid border-red !font-medium" style="--_bg-color: var(--color-red-highlight); --_color: var(--color-text-primary);">Red</TagItem>
				<TagItem 	class="border !border-solid border-orange !font-medium" style="--_bg-color: var(--color-orange-highlight); --_color: var(--color-text-primary);">Orange</TagItem>
				<TagItem 	class="border !border-solid border-blue !font-medium" style="--_bg-color: var(--color-blue-highlight); --_color: var(--color-text-primary);">Blue</TagItem>
				<TagItem 	class="border !border-solid border-purple !font-medium" style="--_bg-color: var(--color-purple-highlight); --_color: var(--color-text-primary);">Purple</TagItem>
				<TagItem 	class="border !border-solid border-gray !font-medium" style="--_bg-color: var(--color-gray-highlight); --_color: var(--color-text-primary);">Gray</TagItem>
			</div>
		`,
	}),
}
