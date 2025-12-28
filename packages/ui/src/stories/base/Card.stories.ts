import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Card from '../../components/base/Card.vue'

const meta = {
	title: 'Base/Card',
	component: Card,
	render: (args) => ({
		components: { Card },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<Card v-bind="args">
				<template #header><h3>Card Title</h3></template>
				<p>This is the card content.</p>
			</Card>
		`,
	}),
} satisfies Meta<typeof Card>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Collapsible: Story = {
	args: {
		collapsible: true,
	},
}

export const CollapsedByDefault: Story = {
	args: {
		collapsible: true,
		defaultCollapsed: true,
	},
}
