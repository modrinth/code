import { PlayIcon, SlashIcon, StopCircleIcon, UpdatedIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import JoinedButtons from '../../components/base/JoinedButtons.vue'

const meta = {
	title: 'Base/JoinedButtons',
	component: JoinedButtons,
	argTypes: {
		color: {
			control: 'select',
			options: ['standard', 'brand', 'red', 'orange', 'green', 'blue', 'purple'],
		},
		size: {
			control: 'select',
			options: ['small', 'standard', 'large'],
		},
		disabled: { control: 'boolean' },
		primaryDisabled: { control: 'boolean' },
		dropdownDisabled: { control: 'boolean' },
		primaryMuted: { control: 'boolean' },
	},
} satisfies Meta<typeof JoinedButtons>

export default meta
type Story = StoryObj<typeof meta>

export const Start: Story = {
	args: {
		color: 'brand',
		size: 'large',
		actions: [
			{
				id: 'start',
				label: 'Start',
				icon: PlayIcon,
				action: () => console.log('Start'),
			},
		],
	},
}

export const StopWithKill: Story = {
	args: {
		color: 'red',
		size: 'large',
		actions: [
			{
				id: 'stop',
				label: 'Stop',
				icon: StopCircleIcon,
				action: () => console.log('Stop'),
			},
			{
				id: 'kill_server',
				label: 'Kill server',
				icon: SlashIcon,
				action: () => console.log('Kill'),
			},
		],
	},
}

export const Stopping: Story = {
	args: {
		color: 'red',
		size: 'large',
		primaryDisabled: true,
		primaryMuted: true,
		actions: [
			{
				id: 'stop',
				label: 'Stopping',
				icon: StopCircleIcon,
				action: () => console.log('Stop'),
			},
			{
				id: 'kill_server',
				label: 'Kill server',
				icon: SlashIcon,
				action: () => console.log('Kill'),
			},
		],
	},
}

export const Restart: Story = {
	args: {
		color: 'orange',
		size: 'large',
		actions: [
			{
				id: 'restart',
				label: 'Restart',
				icon: UpdatedIcon,
				action: () => console.log('Restart'),
			},
		],
	},
}

export const Disabled: Story = {
	args: {
		color: 'red',
		size: 'large',
		disabled: true,
		actions: [
			{
				id: 'stop',
				label: 'Stop',
				icon: StopCircleIcon,
				action: () => console.log('Stop'),
			},
			{
				id: 'kill_server',
				label: 'Kill server',
				icon: SlashIcon,
				action: () => console.log('Kill'),
			},
		],
	},
}
