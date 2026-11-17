import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import Field from '../../components/base/inputs/Field.vue'
import Input from '../../components/base/inputs/Input.vue'

const meta = {
	title: 'Inputs/Field',
	component: Field,
	args: {
		label: 'Name',
		inputId: 'example-name',
	},
} satisfies Meta<typeof Field>

export default meta

export const WithCharacterCountAndHelp: StoryObj<typeof meta> = {
	render: (args) => ({
		components: { Field, Input },
		setup: () => ({ args, name: ref('') }),
		template: `
			<Field v-bind="args" class="max-w-sm">
				<template #right>
					<span class="text-xs text-secondary">{{ name.length }} / 64</span>
				</template>
				<Input :id="args.inputId" v-model="name" :maxlength="64" />
				<template #bottom>
					<p class="mb-0 mt-2 text-sm text-secondary">This name appears on your project page.</p>
				</template>
			</Field>
		`,
	}),
}
