import type { Meta, StoryObj } from '@storybook/vue3-vite'

import SmartClickable from '../../components/base/SmartClickable.vue'

const meta = {
	title: 'Base/SmartClickable',
	component: SmartClickable,
	render: (args) => ({
		components: { SmartClickable },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<SmartClickable v-bind="args">
				<template #clickable>
					<a href="#" style="display: block; width: 100%; height: 100%;"></a>
				</template>
				<div style="padding: 1rem; background: var(--color-button-bg); border-radius: 0.5rem;">
					<h3>Clickable Card</h3>
					<p>The entire card is clickable</p>
				</div>
			</SmartClickable>
		`,
	}),
} satisfies Meta<typeof SmartClickable>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
