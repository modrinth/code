import { SettingsIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Button from '../../components/base/buttons/Button.vue'
import TeleportPopoutMenu from '../../components/base/buttons/TeleportPopoutMenu.vue'

const meta = {
	title: 'Buttons/Teleport Popout Menu',
	component: TeleportPopoutMenu,
} satisfies Meta<typeof TeleportPopoutMenu>

export default meta
type Story = StoryObj<typeof meta>

export const ArbitraryContent: Story = {
	render: () => ({
		components: { Button, SettingsIcon, TeleportPopoutMenu },
		template: /*html*/ `
			<TeleportPopoutMenu label="Configure versions" placement="bottom-start">
				<template #trigger>
					<SettingsIcon />Configure
				</template>

				<template #panel="{ close }">
					<div class="flex w-72 flex-col gap-4">
						<label class="flex flex-col gap-2 font-semibold text-contrast">
							Version name
							<input class="rounded-lg bg-surface-4 px-3 py-2 text-primary ring-1 ring-surface-5" value="1.21.8" />
						</label>
						<Button variant="colored" @click="close()">Apply</Button>
					</div>
				</template>
			</TeleportPopoutMenu>
		`,
	}),
}

export const IconTrigger: Story = {
	render: () => ({
		components: { SettingsIcon, TeleportPopoutMenu },
		template: /*html*/ `
			<TeleportPopoutMenu label="Open settings" variant="quiet" icon-only>
				<template #trigger><SettingsIcon /></template>
				<template #panel>
					<p class="m-0 w-64">Arbitrary teleported content can live here.</p>
				</template>
			</TeleportPopoutMenu>
		`,
	}),
}
