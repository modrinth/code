import { DownloadIcon, ExternalIcon, HeartIcon, SettingsIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Button from '../../components/base/buttons/Button.vue'
import ButtonLink from '../../components/base/buttons/ButtonLink.vue'
import IconButton from '../../components/base/buttons/IconButton.vue'

const variants = ['base', 'colored', 'outlined', 'quiet'] as const
const sizes = ['sm', 'default', 'md', 'lg'] as const
const tones = ['brand', 'red', 'orange', 'green', 'blue', 'purple', 'promotion'] as const
const sizeColumns = [
	{ value: 'sm', label: 'Small' },
	{ value: 'default', label: 'Default' },
	{ value: 'md', label: 'Medium' },
	{ value: 'lg', label: 'Large' },
] as const
const variantRows = [
	{ label: 'Base', variant: 'base' },
	{ label: 'Outlined', variant: 'outlined' },
	{ label: 'Quiet', variant: 'quiet' },
	...tones.map((tone) => ({
		label: `Colored / ${tone.charAt(0).toUpperCase()}${tone.slice(1)}`,
		variant: 'colored' as const,
		tone,
	})),
	...tones.map((tone) => ({
		label: `Quiet / ${tone.charAt(0).toUpperCase()}${tone.slice(1)}`,
		variant: 'quiet' as const,
		tone,
	})),
]

const meta = {
	title: 'Buttons/Button',
	component: Button,
	argTypes: {
		variant: {
			control: 'select',
			options: variants,
		},
		size: {
			control: 'select',
			options: sizes,
		},
		tone: {
			control: 'select',
			options: tones,
		},
		type: {
			control: 'select',
			options: ['button', 'submit', 'reset'],
		},
		disabled: { control: 'boolean' },
		loading: { control: 'boolean' },
	},
	args: {
		variant: 'base',
		size: 'default',
		tone: 'brand',
		type: 'button',
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

export const AllVariants: Story = {
	render: () => ({
		components: { Button, DownloadIcon },
		setup() {
			return { sizeColumns, variantRows }
		},
		template: /*html*/ `
			<div class="grid grid-cols-[max-content_repeat(4,max-content)] items-center gap-4 overflow-x-auto p-1">
				<div />
				<div v-for="size in sizeColumns" :key="size.value" class="font-semibold text-contrast">
					{{ size.label }}
				</div>

				<template v-for="row in variantRows" :key="row.label">
					<div class="whitespace-nowrap font-semibold text-secondary">{{ row.label }}</div>
					<Button
						v-for="size in sizeColumns"
						:key="size.value"
						:variant="row.variant"
						:tone="row.tone"
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
				<Button variant="quiet"><DownloadIcon />Quiet</Button>
				<Button variant="quiet" tone="red"><DownloadIcon />Quiet destructive</Button>
				<IconButton label="Settings" variant="quiet"><SettingsIcon /></IconButton>
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

export const ColoredTones: Story = {
	render: () => ({
		components: { Button },
		setup() {
			return { tones }
		},
		template: /*html*/ `
			<div class="flex flex-wrap items-center gap-4">
				<Button v-for="tone in tones" :key="tone" variant="colored" :tone="tone">
					{{ tone }}
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
				<Button variant="colored">Colored</Button>
				<Button variant="colored" disabled>Colored disabled</Button>
				<Button variant="outlined">Outlined</Button>
				<Button variant="quiet">Quiet</Button>
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
				<ButtonLink href="https://modrinth.com" target="_blank" variant="outlined">
					Modrinth<ExternalIcon />
				</ButtonLink>
				<ButtonLink href="https://modrinth.com" disabled>Disabled link</ButtonLink>
				<IconButton label="Favorite"><HeartIcon /></IconButton>
				<IconButton label="Favorite" variant="colored"><HeartIcon /></IconButton>
				<IconButton label="Favorite" variant="outlined"><HeartIcon /></IconButton>
				<IconButton label="Favorite" variant="quiet"><HeartIcon /></IconButton>
			</div>
		`,
	}),
}
