import type { Meta, StoryObj } from '@storybook/vue3-vite'

import IconSelect from '../IconSelect.vue'

const meta = {
	title: 'Base/IconSelect',
	component: IconSelect,
} satisfies Meta<typeof IconSelect>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {},
	render: () => ({
		components: { IconSelect },
		setup() {
			const icon = undefined
			return { icon }
		},
		template: `
			<IconSelect
				v-model="icon"
				@select="() => console.log('Select icon')"
				@remove="() => console.log('Remove icon')"
			/>
		`,
	}),
}

export const WithIcon: Story = {
	args: {},
	render: () => ({
		components: { IconSelect },
		setup() {
			const icon = 'https://cdn.modrinth.com/data/AANobbMI/icon.png'
			return { icon }
		},
		template: `
			<IconSelect
				v-model="icon"
				@select="() => console.log('Select icon')"
				@remove="() => console.log('Remove icon')"
			/>
		`,
	}),
}
