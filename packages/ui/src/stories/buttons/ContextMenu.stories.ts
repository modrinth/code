import { CopyIcon, FolderOpenIcon, PlayIcon, SettingsIcon, TrashIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'
import { nextTick, onMounted, ref } from 'vue'

import ContextMenu from '../../components/base/buttons/ContextMenu.vue'
import type { ButtonMenuOption } from '../../components/base/buttons/types'

const options: ButtonMenuOption[] = [
	{
		id: 'play',
		label: 'Play',
		icon: PlayIcon,
		tone: 'brand',
		action: () => undefined,
	},
	{
		id: 'copy',
		label: 'Copy',
		icon: CopyIcon,
		type: 'submenu',
		options: [
			{ id: 'copy_name', label: 'Copy name', icon: CopyIcon, action: () => undefined },
			{ id: 'copy_path', label: 'Copy path', icon: CopyIcon, action: () => undefined },
			{ id: 'copy_id', label: 'Copy ID', icon: CopyIcon, action: () => undefined },
		],
	},
	{ id: 'open_folder', label: 'Open folder', icon: FolderOpenIcon, action: () => undefined },
	{ type: 'divider' },
	{ id: 'settings', label: 'Settings', icon: SettingsIcon, action: () => undefined },
	{
		id: 'delete',
		label: 'Delete',
		icon: TrashIcon,
		tone: 'red',
		hoverFilledOnly: true,
		action: () => undefined,
	},
]

const meta = {
	title: 'Buttons/Context Menu',
	component: ContextMenu,
	parameters: {
		layout: 'fullscreen',
	},
	args: {
		label: 'Instance actions',
		onSelect: fn(),
		onOpen: fn(),
		onClose: fn(),
	},
	render: (args) => ({
		components: { ContextMenu },
		setup() {
			const contextMenu = ref<InstanceType<typeof ContextMenu>>()
			const target = ref<HTMLElement>()

			function openMenu(event: MouseEvent) {
				contextMenu.value?.open(event, options)
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
					:label="args.label"
					@select="args.onSelect"
					@open="args.onOpen"
					@close="args.onClose"
				/>
			</div>
		`,
	}),
} satisfies Meta<typeof ContextMenu>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
