import type { Meta, StoryObj } from '@storybook/vue3-vite'

import RadialHeader from '../../components/base/RadialHeader.vue'

const meta = {
	title: 'Base/RadialHeader',
	component: RadialHeader,
} satisfies Meta<typeof RadialHeader>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { RadialHeader },
		template: `
			<RadialHeader class="p-8">
				<h1 class="text-2xl font-bold text-center">Radial Header Content</h1>
			</RadialHeader>
		`,
	}),
}

export const AllColors: Story = {
	render: () => ({
		components: { RadialHeader },
		template: `
			<div class="flex flex-col gap-4">
				<RadialHeader color="brand" class="p-8">
					<p class="text-center font-semibold">Brand</p>
				</RadialHeader>
				<RadialHeader color="red" class="p-8">
					<p class="text-center font-semibold">Red</p>
				</RadialHeader>
				<RadialHeader color="orange" class="p-8">
					<p class="text-center font-semibold">Orange</p>
				</RadialHeader>
				<RadialHeader color="green" class="p-8">
					<p class="text-center font-semibold">Green</p>
				</RadialHeader>
				<RadialHeader color="blue" class="p-8">
					<p class="text-center font-semibold">Blue</p>
				</RadialHeader>
				<RadialHeader color="purple" class="p-8">
					<p class="text-center font-semibold">Purple</p>
				</RadialHeader>
				<RadialHeader color="gray" class="p-8">
					<p class="text-center font-semibold">Gray</p>
				</RadialHeader>
			</div>
		`,
	}),
}
