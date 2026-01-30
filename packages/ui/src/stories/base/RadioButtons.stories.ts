import type { Meta, StoryObj } from '@storybook/vue3-vite'

import RadioButtons from '../../components/base/RadioButtons.vue'

const meta = {
	title: 'Base/RadioButtons',
	// @ts-ignore - error comes from generically typed component
	component: RadioButtons,
	render: (args) => ({
		components: { RadioButtons },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<RadioButtons v-bind="args" v-model="args.modelValue">
				<template #default="{ item }">{{ item }}</template>
			</RadioButtons>
		`,
	}),
} satisfies Meta<typeof RadioButtons>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		items: ['Option 1', 'Option 2', 'Option 3'],
		modelValue: 'Option 1',
	},
}

export const ManyOptions: Story = {
	args: {
		items: ['Daily', 'Weekly', 'Monthly', 'Yearly'],
		modelValue: 'Weekly',
	},
}
