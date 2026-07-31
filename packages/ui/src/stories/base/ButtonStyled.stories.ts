import { DownloadIcon, HeartIcon, SettingsIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ButtonStyled from '../../components/base/ButtonStyled.vue'

const colors = ['standard', 'brand', 'red', 'orange', 'green', 'blue', 'purple'] as const
const types = [
	'standard',
	'outlined',
	'transparent',
	'highlight',
	'highlight-colored-text',
	'chip',
] as const
const sizes = ['small', 'standard', 'large'] as const

const meta = {
	title: 'Base/ButtonStyled',
	component: ButtonStyled,
	argTypes: {
		color: {
			control: 'select',
			options: [...colors, 'medal-promo'],
		},
		size: {
			control: 'select',
			options: [...sizes],
		},
		type: {
			control: 'select',
			options: [...types],
		},
		circular: { control: 'boolean' },
		colorFill: {
			control: 'select',
			options: ['auto', 'background', 'text', 'none'],
		},
		hoverColorFill: {
			control: 'select',
			options: ['auto', 'background', 'text', 'none'],
		},
		highlighted: { control: 'boolean' },
		highlightedStyle: {
			control: 'select',
			options: ['main-nav-primary', 'main-nav-secondary'],
		},
	},
	args: {
		color: 'standard',
		size: 'standard',
		type: 'standard',
		circular: false,
		colorFill: 'auto',
		hoverColorFill: 'auto',
		highlighted: false,
		highlightedStyle: 'main-nav-primary',
	},
	render: (args) => ({
		components: { ButtonStyled, DownloadIcon },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<ButtonStyled v-bind="args">
				<button><DownloadIcon /> Button</button>
			</ButtonStyled>
		`,
	}),
} satisfies Meta<typeof ButtonStyled>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		type: 'standard',
	},
}

export const AllVariants: Story = {
	render: () => ({
		components: { ButtonStyled },
		setup() {
			return { colors, types }
		},
		template: /*html*/ `
			<div style="overflow-x: auto;">
				<table style="border-collapse: collapse; width: 100%;">
					<thead>
						<tr>
							<th style="padding: 0.5rem 1rem; text-align: left; font-size: 0.75rem; text-transform: uppercase; color: var(--color-secondary);">Color / Type</th>
							<th v-for="type in types" :key="type" style="padding: 0.5rem 1rem; text-align: center; font-size: 0.75rem; text-transform: uppercase; color: var(--color-secondary);">{{ type }}</th>
						</tr>
					</thead>
					<tbody>
						<tr v-for="color in colors" :key="color" style="border-top: 1px solid var(--color-surface-5);">
							<td style="padding: 0.5rem 1rem; font-weight: 600; font-size: 0.875rem;">{{ color }}</td>
							<td v-for="type in types" :key="type" style="padding: 0.5rem 1rem; text-align: center;">
								<ButtonStyled :color="color" :type="type">
									<button>Button</button>
								</ButtonStyled>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		`,
	}),
}

export const AllVariantsHighlighted: Story = {
	render: () => ({
		components: { ButtonStyled },
		setup() {
			return { colors, types }
		},
		template: /*html*/ `
			<div style="overflow-x: auto;">
				<table style="border-collapse: collapse; width: 100%;">
					<thead>
						<tr>
							<th style="padding: 0.5rem 1rem; text-align: left; font-size: 0.75rem; text-transform: uppercase; color: var(--color-secondary);">Color / Type</th>
							<th v-for="type in types" :key="type" style="padding: 0.5rem 1rem; text-align: center; font-size: 0.75rem; text-transform: uppercase; color: var(--color-secondary);">{{ type }}</th>
						</tr>
					</thead>
					<tbody>
						<tr v-for="color in colors" :key="color" style="border-top: 1px solid var(--color-surface-5);">
							<td style="padding: 0.5rem 1rem; font-weight: 600; font-size: 0.875rem;">{{ color }}</td>
							<td v-for="type in types" :key="type" style="padding: 0.5rem 1rem; text-align: center;">
								<ButtonStyled :color="color" :type="type" highlighted>
									<button>Button</button>
								</ButtonStyled>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		`,
	}),
}

export const Sizes: Story = {
	render: () => ({
		components: { ButtonStyled },
		setup() {
			return { sizes, types }
		},
		template: /*html*/ `
			<div style="overflow-x: auto;">
				<table style="border-collapse: collapse; width: 100%;">
					<thead>
						<tr>
							<th style="padding: 0.5rem 1rem; text-align: left; font-size: 0.75rem; text-transform: uppercase; color: var(--color-secondary);">Size / Type</th>
							<th v-for="type in types" :key="type" style="padding: 0.5rem 1rem; text-align: center; font-size: 0.75rem; text-transform: uppercase; color: var(--color-secondary);">{{ type }}</th>
						</tr>
					</thead>
					<tbody>
						<tr v-for="size in sizes" :key="size" style="border-top: 1px solid var(--color-surface-5);">
							<td style="padding: 0.5rem 1rem; font-weight: 600; font-size: 0.875rem;">{{ size }}</td>
							<td v-for="type in types" :key="type" style="padding: 0.5rem 1rem; text-align: center;">
								<ButtonStyled :size="size" :type="type" color="brand">
									<button>Button</button>
								</ButtonStyled>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		`,
	}),
}

export const WithIcons: Story = {
	render: () => ({
		components: { ButtonStyled, DownloadIcon, HeartIcon, SettingsIcon },
		setup() {
			return { types }
		},
		template: /*html*/ `
			<div style="overflow-x: auto;">
				<table style="border-collapse: collapse; width: 100%;">
					<thead>
						<tr>
							<th style="padding: 0.5rem 1rem; text-align: left; font-size: 0.75rem; text-transform: uppercase; color: var(--color-secondary);">Variant</th>
							<th v-for="type in types" :key="type" style="padding: 0.5rem 1rem; text-align: center; font-size: 0.75rem; text-transform: uppercase; color: var(--color-secondary);">{{ type }}</th>
						</tr>
					</thead>
					<tbody>
						<tr style="border-top: 1px solid var(--color-surface-5);">
							<td style="padding: 0.5rem 1rem; font-weight: 600; font-size: 0.875rem;">Icon + text</td>
							<td v-for="type in types" :key="type" style="padding: 0.5rem 1rem; text-align: center;">
								<ButtonStyled :type="type" color="brand">
									<button><DownloadIcon /> Download</button>
								</ButtonStyled>
							</td>
						</tr>
						<tr style="border-top: 1px solid var(--color-surface-5);">
							<td style="padding: 0.5rem 1rem; font-weight: 600; font-size: 0.875rem;">Icon only</td>
							<td v-for="type in types" :key="type" style="padding: 0.5rem 1rem; text-align: center;">
								<ButtonStyled :type="type" color="brand" circular>
									<button><HeartIcon /></button>
								</ButtonStyled>
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		`,
	}),
}

export const Disabled: Story = {
	render: () => ({
		components: { ButtonStyled },
		setup() {
			return { types }
		},
		template: /*html*/ `
			<div style="display: flex; flex-wrap: wrap; gap: 1rem; align-items: center;">
				<ButtonStyled v-for="type in types" :key="type" :type="type" color="brand">
					<button disabled>{{ type }}</button>
				</ButtonStyled>
			</div>
		`,
	}),
}
