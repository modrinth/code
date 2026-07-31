import { MoreHorizontalIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import OverflowMenu from '../../components/base/OverflowMenu.vue'

const meta = {
	title: 'Base/OverflowMenu',
	component: OverflowMenu,
	render: (args) => ({
		components: { OverflowMenu, MoreHorizontalIcon, ButtonStyled },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<ButtonStyled circular type="transparent">
				<OverflowMenu v-bind="args">
					<MoreHorizontalIcon class="h-5 w-5" />
					<template #edit>Edit</template>
					<template #delete>Delete</template>
					<template #share>Share</template>
				</OverflowMenu>
			</ButtonStyled>
		`,
	}),
} satisfies Meta<typeof OverflowMenu>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		options: [
			{ id: 'edit', action: () => console.log('Edit clicked') },
			{ id: 'share', action: () => console.log('Share clicked') },
			{ divider: true },
			{ id: 'delete', action: () => console.log('Delete clicked'), color: 'danger' },
		],
	},
}

export const WithDifferentPlacements: StoryObj = {
	render: () => ({
		components: { OverflowMenu, MoreHorizontalIcon, ButtonStyled },
		template: /*html*/ `
			<div class="flex gap-8 items-start">
				<div class="flex flex-col items-center gap-2">
					<span class="text-sm text-secondary">bottom-end (default)</span>
					<ButtonStyled circular type="transparent">
						<OverflowMenu
							:options="[
								{ id: 'edit', action: () => {} },
								{ id: 'delete', action: () => {}, color: 'danger' },
							]"
						>
							<MoreHorizontalIcon class="h-5 w-5" />
							<template #edit>Edit</template>
							<template #delete>Delete</template>
						</OverflowMenu>
					</ButtonStyled>
				</div>
				<div class="flex flex-col items-center gap-2">
					<span class="text-sm text-secondary">bottom-start</span>
					<ButtonStyled circular type="transparent">
						<OverflowMenu
							direction="left"
							:options="[
								{ id: 'edit', action: () => {} },
								{ id: 'delete', action: () => {}, color: 'danger' },
							]"
						>
							<MoreHorizontalIcon class="h-5 w-5" />
							<template #edit>Edit</template>
							<template #delete>Delete</template>
						</OverflowMenu>
					</ButtonStyled>
				</div>
			</div>
		`,
	}),
}
