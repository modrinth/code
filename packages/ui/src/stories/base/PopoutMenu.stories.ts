import { SettingsIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Button from '../../components/base/Button.vue'
import ButtonStyled from '../../components/base/ButtonStyled.vue'
import PopoutMenu from '../../components/base/PopoutMenu.vue'

const meta = {
	title: 'Base/PopoutMenu',
	component: PopoutMenu,
	render: (args) => ({
		components: { PopoutMenu, Button, ButtonStyled, SettingsIcon },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<ButtonStyled circular type="transparent">
				<PopoutMenu v-bind="args">
					<SettingsIcon class="h-5 w-5" />
					<template #menu>
						<div class="flex flex-col gap-1 p-1">
							<Button transparent>Option 1</Button>
							<Button transparent>Option 2</Button>
							<Button transparent>Option 3</Button>
						</div>
					</template>
				</PopoutMenu>
			</ButtonStyled>
		`,
	}),
} satisfies Meta<typeof PopoutMenu>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithTooltip: Story = {
	args: {
		tooltip: 'Click for more options',
	},
}

export const DifferentPlacements: StoryObj = {
	render: () => ({
		components: { PopoutMenu, Button, ButtonStyled, SettingsIcon },
		template: /*html*/ `
			<div class="flex gap-8 items-start p-8">
				<div class="flex flex-col items-center gap-2">
					<span class="text-sm text-secondary">bottom-end (default)</span>
					<ButtonStyled circular type="transparent">
						<PopoutMenu placement="bottom-end">
							<SettingsIcon class="h-5 w-5" />
							<template #menu>
								<div class="flex flex-col gap-1 p-1">
									<Button transparent>Option 1</Button>
									<Button transparent>Option 2</Button>
								</div>
							</template>
						</PopoutMenu>
					</ButtonStyled>
				</div>
				<div class="flex flex-col items-center gap-2">
					<span class="text-sm text-secondary">bottom-start</span>
					<ButtonStyled circular type="transparent">
						<PopoutMenu placement="bottom-start">
							<SettingsIcon class="h-5 w-5" />
							<template #menu>
								<div class="flex flex-col gap-1 p-1">
									<Button transparent>Option 1</Button>
									<Button transparent>Option 2</Button>
								</div>
							</template>
						</PopoutMenu>
					</ButtonStyled>
				</div>
				<div class="flex flex-col items-center gap-2">
					<span class="text-sm text-secondary">top-end</span>
					<ButtonStyled circular type="transparent">
						<PopoutMenu placement="top-end">
							<SettingsIcon class="h-5 w-5" />
							<template #menu>
								<div class="flex flex-col gap-1 p-1">
									<Button transparent>Option 1</Button>
									<Button transparent>Option 2</Button>
								</div>
							</template>
						</PopoutMenu>
					</ButtonStyled>
				</div>
			</div>
		`,
	}),
}
