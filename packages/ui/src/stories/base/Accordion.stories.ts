import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Accordion from '../Accordion.vue'

const meta = {
	title: 'Base/Accordion',
	component: Accordion,
	render: (args) => ({
		components: { Accordion },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<Accordion v-bind="args">
				<template #title>Click to expand</template>
				<p>This is the accordion content that shows when expanded.</p>
			</Accordion>
		`,
	}),
} satisfies Meta<typeof Accordion>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const OpenByDefault: Story = {
	args: {
		openByDefault: true,
	},
}

export const ForceOpen: Story = {
	args: {
		forceOpen: true,
	},
}
