import { CopyIcon, FolderOpenIcon, PlayIcon, SettingsIcon, TrashIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'
import { nextTick, onMounted, ref } from 'vue'

import ContextMenu from '../../../../../apps/app-frontend/src/components/ui/context-menu/index.vue'
import type { ContextMenuOption } from '../../../../../apps/app-frontend/src/components/ui/context-menu/types'

const options: ContextMenuOption[] = [
	{ name: 'play', color: 'primary' },
	{
		name: 'copy',
		children: [{ name: 'copy_name' }, { name: 'copy_path' }, { name: 'copy_id' }],
	},
	{ name: 'open_folder' },
	{ type: 'divider' },
	{ name: 'settings' },
	{ name: 'delete', color: 'danger' },
]

const meta = {
	title: 'App/Context Menu',
	component: ContextMenu,
	parameters: {
		layout: 'fullscreen',
	},
	args: {
		onMenuClosed: fn(),
		onOptionClicked: fn(),
	},
	render: (args) => ({
		components: {
			ContextMenu,
			CopyIcon,
			FolderOpenIcon,
			PlayIcon,
			SettingsIcon,
			TrashIcon,
		},
		setup() {
			const contextMenu = ref<InstanceType<typeof ContextMenu>>()
			const target = ref<HTMLElement>()
			const item = { id: 'storybook-instance', name: 'Storybook Instance' }

			function openMenu(event: MouseEvent) {
				contextMenu.value?.showMenu(event, item, options)
			}

			onMounted(() => {
				nextTick(() => {
					const rect = target.value?.getBoundingClientRect()
					openMenu(
						new MouseEvent('contextmenu', {
							clientX: (rect?.left ?? 80) + 80,
							clientY: (rect?.top ?? 80) + 80,
						}),
					)
				})
			})

			return { args, contextMenu, openMenu, target }
		},
		template: /*html*/ `
			<div
				ref="target"
				style="box-sizing: border-box; min-height: 100vh; padding: 5rem; background: var(--color-bg); color: var(--color-text-primary);"
				@contextmenu.prevent.stop="openMenu"
			>
				<div style="max-width: 32rem; border: 1px dashed var(--color-divider); border-radius: var(--radius-lg); padding: 2rem;">
					<p style="margin: 0; color: var(--color-text-secondary);">
						Right-click anywhere in this panel to reopen the menu.
					</p>
				</div>

				<ContextMenu
					ref="contextMenu"
					@menu-closed="args.onMenuClosed"
					@option-clicked="args.onOptionClicked"
				>
					<template #play><PlayIcon /> Play</template>
					<template #copy><CopyIcon /> Copy</template>
					<template #copy_name><CopyIcon /> Copy name</template>
					<template #copy_path><CopyIcon /> Copy path</template>
					<template #copy_id> <CopyIcon /> Copy ID</template>
					<template #open_folder><FolderOpenIcon /> Open folder</template>
					<template #settings><SettingsIcon /> Settings</template>
					<template #delete><TrashIcon /> Delete</template>
				</ContextMenu>
			</div>
		`,
	}),
} satisfies Meta<typeof ContextMenu>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
