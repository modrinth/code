import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import type { MemberWeight } from './allocation'
import RevenueSplitEditor from './index.vue'

const roster = [
	{ name: 'Halcyon', color: 'var(--color-brand)' },
	{ name: 'JuniperBuilds', color: 'var(--color-blue)' },
	{ name: 'OllieCodes', color: 'var(--color-purple)' },
	{ name: 'Mossworks', color: 'var(--color-orange)' },
	{ name: 'Finch', color: 'var(--color-red)' },
	{ name: 'NovaCircuit', color: '#2cbab5' },
	{ name: 'PixelMoth', color: '#ec87bd' },
	{ name: 'Cinder', color: '#d8b34a' },
	{ name: 'Riverstone', color: '#8499ed' },
	{ name: 'Fern', color: '#93b85c' },
]

function weights(values: number[], organizationMembers: number[] = []): MemberWeight[] {
	return values.map((percentage, index) => ({
		...roster[index],
		id: String(index + 1),
		percentage,
		organizationMember: organizationMembers.includes(index),
		fixed: organizationMembers.includes(index),
	}))
}

const meta = {
	title: 'Website/Project Settings/Revenue Split Editor',
	component: RevenueSplitEditor,
	args: { modelValue: weights([60, 25, 15], [0]), readOnly: false },
	render: (args) => ({
		components: { RevenueSplitEditor },
		setup: () => ({ args, members: ref(args.modelValue.map((member) => ({ ...member }))) }),
		template:
			'<div class="mx-auto max-w-3xl"><RevenueSplitEditor v-model="members" :read-only="args.readOnly" /></div>',
	}),
} satisfies Meta<typeof RevenueSplitEditor>

export default meta
type Story = StoryObj<typeof meta>

export const OneMember: Story = {
	name: '1 member',
	args: { modelValue: weights([100]), readOnly: true },
}
export const TwoMembers: Story = { name: '2 members', args: { modelValue: weights([60, 40]) } }
export const ThreeMembers: Story = { name: '3 members' }
export const FiveMembers: Story = {
	name: '5 members',
	args: { modelValue: weights([33.33, 21.25, 10, 8.5, 0], [0, 2]) },
}
export const TenMembers: Story = {
	name: '10 members',
	args: { modelValue: weights([24, 16, 12, 8.25, 8, 6, 5.75, 4.5, 3, 0], [0, 5]) },
}
export const ReadOnly: Story = { args: { readOnly: true, modelValue: weights([60, 25, 15]) } }
