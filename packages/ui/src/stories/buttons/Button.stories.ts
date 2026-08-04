import { DownloadIcon, ExternalIcon, HeartIcon, SettingsIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Button from '../../components/base/buttons/Button.vue'
import ButtonLink from '../../components/base/buttons/ButtonLink.vue'
import IconButton from '../../components/base/buttons/IconButton.vue'

const types = ['base', 'colored', 'outlined', 'quiet'] as const
const sizes = ['sm', 'md', 'lg', 'xl'] as const
const colors = ['brand', 'red', 'orange', 'green', 'blue', 'purple', 'medal_promotion'] as const
const sizeColumns = [
	{ value: 'sm', label: 'Small' },
	{ value: 'md', label: 'Medium' },
	{ value: 'lg', label: 'Large' },
	{ value: 'xl', label: 'Extra large' },
] as const
const typeRows = [
	{ label: 'Base', type: 'base' },
	{ label: 'Outlined', type: 'outlined' },
	{ label: 'Quiet', type: 'quiet' },
	...colors.map((color) => ({
		label: `Colored / ${color.charAt(0).toUpperCase()}${color.slice(1)}`,
		type: 'colored' as const,
		color,
	})),
	...colors.map((color) => ({
		label: `Outlined / ${color.charAt(0).toUpperCase()}${color.slice(1)}`,
		type: 'outlined' as const,
		color,
	})),
	...colors.map((color) => ({
		label: `Quiet / ${color.charAt(0).toUpperCase()}${color.slice(1)}`,
		type: 'quiet' as const,
		color,
	})),
]

const meta = {
	title: 'Buttons/Button',
	component: Button,
	argTypes: {
		type: {
			control: 'select',
			options: types,
		},
		size: {
			control: 'select',
			options: sizes,
		},
		color: {
			control: 'select',
			options: colors,
		},
		nativeType: {
			control: 'select',
			options: ['button', 'submit', 'reset'],
		},
		disabled: { control: 'boolean' },
		loading: { control: 'boolean' },
	},
	args: {
		type: 'base',
		size: 'md',
		color: 'brand',
		nativeType: 'button',
		disabled: false,
		loading: false,
	},
	render: (args) => ({
		components: { Button, DownloadIcon },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<Button v-bind="args">
				<DownloadIcon />
				Download
			</Button>
		`,
	}),
} satisfies Meta<typeof Button>

export default meta
type Story = StoryObj<typeof meta>

export const Playground: Story = {}

export const AllTypes: Story = {
	render: () => ({
		components: { Button, DownloadIcon },
		setup() {
			return { sizeColumns, typeRows }
		},
		template: /*html*/ `
			<div class="grid grid-cols-[max-content_repeat(4,max-content)] items-center gap-4 overflow-x-auto p-1">
				<div />
				<div v-for="size in sizeColumns" :key="size.value" class="font-semibold text-contrast">
					{{ size.label }}
				</div>

				<template v-for="row in typeRows" :key="row.label">
					<div class="whitespace-nowrap font-semibold text-secondary">{{ row.label }}</div>
					<Button
						v-for="size in sizeColumns"
						:key="size.value"
						:type="row.type"
						:color="row.color"
						:size="size.value"
					>
						<DownloadIcon />Button
					</Button>
				</template>
			</div>
		`,
	}),
}

export const Quiet: Story = {
	render: () => ({
		components: { Button, DownloadIcon, IconButton, SettingsIcon },
		template: /*html*/ `
			<div class="flex flex-wrap items-center gap-4">
				<Button type="quiet"><DownloadIcon />Quiet</Button>
				<Button type="quiet" color="red"><DownloadIcon />Quiet destructive</Button>
				<IconButton label="Settings" type="quiet"><SettingsIcon /></IconButton>
			</div>
		`,
	}),
}

export const Sizes: Story = {
	render: () => ({
		components: { Button, DownloadIcon, IconButton },
		setup() {
			return { sizes }
		},
		template: /*html*/ `
			<div class="flex flex-wrap items-center gap-4">
				<template v-for="size in sizes" :key="size">
					<Button :size="size"><DownloadIcon />{{ size }}</Button>
					<IconButton :label="size" :size="size"><DownloadIcon /></IconButton>
				</template>
			</div>
		`,
	}),
}

export const Colors: Story = {
	render: () => ({
		components: { Button },
		setup() {
			return { colors }
		},
		template: /*html*/ `
			<div class="flex flex-wrap items-center gap-4">
				<Button v-for="color in colors" :key="color" type="colored" :color="color">
					{{ color }}
				</Button>
			</div>
		`,
	}),
}

export const Content: Story = {
	render: () => ({
		components: { Button, DownloadIcon, SettingsIcon },
		template: /*html*/ `
			<div class="flex max-w-2xl flex-col items-start gap-4">
				<Button>Text only</Button>
				<Button><DownloadIcon />Leading icon</Button>
				<Button>Trailing icon<SettingsIcon /></Button>
				<Button class="w-full">Full width</Button>
				<Button>Continue with a deliberately long translated action label</Button>
			</div>
		`,
	}),
}

export const InteractionStates: Story = {
	render: () => ({
		components: { Button },
		template: /*html*/ `
			<div class="flex flex-wrap items-center gap-4">
				<Button>Enabled</Button>
				<Button disabled>Disabled</Button>
				<Button loading>Loading</Button>
				<Button type="colored">Colored</Button>
				<Button type="colored" disabled>Colored disabled</Button>
				<Button type="outlined">Outlined</Button>
				<Button type="quiet">Quiet</Button>
			</div>
		`,
	}),
}

export const LinksAndIconButton: Story = {
	render: () => ({
		components: { ButtonLink, ExternalIcon, HeartIcon, IconButton },
		template: /*html*/ `
			<div class="flex flex-wrap items-center gap-4">
				<ButtonLink to="/library">Internal link</ButtonLink>
				<ButtonLink href="https://modrinth.com" target="_blank" type="outlined">
					Modrinth<ExternalIcon />
				</ButtonLink>
				<ButtonLink href="https://modrinth.com" disabled>Disabled link</ButtonLink>
				<IconButton label="Favorite"><HeartIcon /></IconButton>
				<IconButton label="Favorite" type="colored"><HeartIcon /></IconButton>
				<IconButton label="Favorite" type="outlined"><HeartIcon /></IconButton>
				<IconButton label="Favorite" type="quiet"><HeartIcon /></IconButton>
			</div>
		`,
	}),
}
