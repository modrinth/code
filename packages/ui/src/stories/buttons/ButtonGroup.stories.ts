import { PlayIcon, SettingsIcon, StopCircleIcon, TrashIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Button from '../../components/base/buttons/Button.vue'
import ButtonGroup from '../../components/base/buttons/ButtonGroup.vue'
import SplitButton from '../../components/base/buttons/SplitButton.vue'
import type { OverflowMenuOption } from '../../components/base/buttons/types'

const splitOptions: OverflowMenuOption[] = [
	{
		id: 'settings',
		label: 'Server settings',
		icon: SettingsIcon,
		action: () => undefined,
	},
	{ type: 'divider' },
	{
		id: 'delete',
		label: 'Delete server',
		icon: TrashIcon,
		tone: 'red',
		action: () => undefined,
	},
]

const meta = {
	title: 'Buttons/Button Group',
	component: ButtonGroup,
} satisfies Meta<typeof ButtonGroup>

export default meta
type Story = StoryObj<typeof meta>

export const Joined: Story = {
	render: () => ({
		components: { Button, ButtonGroup },
		template: /*html*/ `
			<ButtonGroup label="Pagination">
				<Button variant="outlined">Previous</Button>
				<Button variant="outlined">Next</Button>
			</ButtonGroup>
		`,
	}),
}

export const Split: Story = {
	render: () => ({
		components: { PlayIcon, SplitButton },
		setup() {
			return { splitOptions }
		},
		template: /*html*/ `
			<SplitButton
				menu-label="More server actions"
				group-label="Server actions"
				variant="colored"
				:options="splitOptions"
			>
				<PlayIcon />Start server
			</SplitButton>
		`,
	}),
}

export const IndependentDisabledStates: Story = {
	render: () => ({
		components: { SplitButton, StopCircleIcon },
		setup() {
			return { splitOptions }
		},
		template: /*html*/ `
			<div class="flex flex-wrap gap-4">
				<SplitButton menu-label="More actions" :options="splitOptions" primary-disabled>
					<StopCircleIcon />Primary disabled
				</SplitButton>
				<SplitButton menu-label="More actions" :options="splitOptions" menu-disabled>
					<StopCircleIcon />Menu disabled
				</SplitButton>
			</div>
		`,
	}),
}
