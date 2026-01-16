import type { Meta, StoryObj } from '@storybook/vue3-vite'

import OptionGroup from '../../components/base/OptionGroup.vue'

const meta = {
	title: 'Base/OptionGroup',
	// @ts-ignore - error comes from generically typed component
	component: OptionGroup,
	render: (args) => ({
		components: { OptionGroup },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<OptionGroup v-bind="args" v-model="args.modelValue">
				<template #default="{ option }">{{ option }}</template>
			</OptionGroup>
		`,
	}),
} satisfies Meta<typeof OptionGroup>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		options: ['Option 1', 'Option 2', 'Option 3'],
		modelValue: 'Option 1',
	},
}

export const ManyOptions: Story = {
	args: {
		options: ['All', 'Mods', 'Plugins', 'Resource Packs', 'Modpacks', 'Shaders'],
		modelValue: 'All',
	},
}
