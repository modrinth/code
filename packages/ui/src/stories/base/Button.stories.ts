import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Button from '../../components/base/Button.vue'

const meta = {
	title: 'Base/Button',
	component: Button,
	render: (args) => ({
		components: { Button },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<Button v-bind="args">Click me</Button>
		`,
	}),
} satisfies Meta<typeof Button>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Primary: Story = {
	args: {
		color: 'primary',
	},
}

export const Danger: Story = {
	args: {
		color: 'danger',
	},
}

export const AllColors: Story = {
	render: () => ({
		components: { Button },
		template: /*html*/ `
			<div style="display: flex; flex-wrap: wrap; gap: 1rem;">
				<Button color="default">Default</Button>
				<Button color="primary">Primary</Button>
				<Button color="danger">Danger</Button>
				<Button color="red">Red</Button>
				<Button color="orange">Orange</Button>
				<Button color="green">Green</Button>
				<Button color="blue">Blue</Button>
				<Button color="purple">Purple</Button>
			</div>
		`,
	}),
}

export const Large: Story = {
	args: {
		large: true,
	},
}

export const Outline: Story = {
	args: {
		outline: true,
	},
}

export const Transparent: Story = {
	args: {
		transparent: true,
	},
}

export const Disabled: Story = {
	args: {
		disabled: true,
	},
}

export const AsLink: Story = {
	args: {
		link: 'https://modrinth.com',
		external: true,
	},
}
