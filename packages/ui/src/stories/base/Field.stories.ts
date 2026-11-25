import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import Field from '../../components/base/Field.vue'
import Input from '../../components/base/inputs/Input.vue'

const meta = {
	title: 'Base/Field',
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
				<template #subtext>
					This name appears on your project page.
				</template>
			</Field>
		`,
	}),
}

export const WithOptionalLabel: StoryObj<typeof meta> = {
	...WithCharacterCountAndHelp,
	args: { labelOptional: true },
}

export const WithCustomSubtext: StoryObj<typeof meta> = {
	...WithCharacterCountAndHelp,
	args: { subtextClass: 'text-sm leading-normal text-secondary' },
}
