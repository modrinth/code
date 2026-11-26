import { GlobeIcon, LinkIcon, LockIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import RadioCard from '../../components/base/RadioCard.vue'

const meta = {
	title: 'Base/RadioCard',
	component: RadioCard,
	args: {
		name: 'visibility',
		modelValue: 'public',
		value: 'public',
		label: 'Public',
		description: 'Visible via URL, on your profile, and in search.',
		icon: GlobeIcon,
	},
} satisfies Meta<typeof RadioCard>
export default meta

type Story = StoryObj<typeof meta>

export const Selected: Story = {}
export const Unselected: Story = { args: { modelValue: 'private' } }
export const Disabled: Story = { args: { disabled: true } }
export const DisabledUnselected: Story = { args: { disabled: true, modelValue: 'private' } }
export const WithoutIcon: Story = { args: { icon: undefined } }
export const WithoutDescription: Story = { args: { description: undefined } }
export const WithMaxWidth: Story = { args: { maxWidth: '250px' } }
export const Group: Story = {
	render: () => ({
		components: { RadioCard },
		setup() {
			return {
				selected: ref('public'),
				options: [
					{
						value: 'public',
						label: 'Public',
						description: 'Visible via URL, on your profile, and in search.',
						icon: GlobeIcon,
					},
					{
						value: 'unlisted',
						label: 'Unlisted',
						description: 'Visible via URL only. Not shown on your profile or in search.',
						icon: LinkIcon,
					},
					{
						value: 'private',
						label: 'Private',
						description: 'Not publicly visible. Only accessible to project members.',
						icon: LockIcon,
					},
				],
			}
		},
		template: `<fieldset class="m-0 max-w-3xl border-0 p-0"><legend class="mb-2 font-semibold text-contrast">Visibility</legend><div class="grid grid-cols-1 gap-2 sm:grid-cols-3"><RadioCard v-for="option in options" :key="option.value" v-model="selected" name="visibility-group" v-bind="option" /></div></fieldset>`,
	}),
}
