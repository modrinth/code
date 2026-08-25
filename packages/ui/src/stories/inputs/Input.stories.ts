import { KeyIcon, LinkIcon, MailIcon, SearchIcon, UserIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import Input from '../../components/base/inputs/Input.vue'

const meta = {
	title: 'Inputs/Input',
	component: Input,
	argTypes: {
		size: {
			control: 'select',
			options: ['small', 'standard', 'medium', 'large'],
		},
		type: {
			control: 'select',
			options: ['text', 'email', 'password', 'number', 'url', 'search'],
		},
	},
} satisfies Meta<typeof Input>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		placeholder: 'Enter text...',
	},
}

export const WithIcon: Story = {
	args: {
		icon: SearchIcon,
		placeholder: 'Search...',
	},
}

export const WithTextPrefix: Story = {
	render: () => ({
		components: { Input },
		setup() {
			const value = ref('sodium')
			return { value }
		},
		template: `
			<Input v-model="value">
				<template #prefix>
					<span style="white-space: nowrap">https://modrinth.com/mod/</span>
				</template>
			</Input>
		`,
	}),
}

export const Clearable: Story = {
	render: () => ({
		components: { Input },
		setup() {
			const value = ref('Some text to clear')
			return { value, SearchIcon }
		},
		template: `
			<Input
				v-model="value"
				:icon="SearchIcon"
				placeholder="Search..."
				clearable
			/>
		`,
	}),
}

export const WithIconAndClearable: Story = {
	render: () => ({
		components: { Input },
		setup() {
			const value = ref('Search query')
			return { value, SearchIcon }
		},
		template: `
			<Input
				v-model="value"
				:icon="SearchIcon"
				placeholder="Search..."
				clearable
			/>
		`,
	}),
}

export const Password: Story = {
	args: {
		icon: KeyIcon,
		type: 'password',
		placeholder: 'Password',
		autocomplete: 'current-password',
	},
}

export const Email: Story = {
	args: {
		icon: MailIcon,
		type: 'email',
		placeholder: 'Email address',
		autocomplete: 'email',
	},
}

export const Search: Story = {
	args: {
		icon: SearchIcon,
		type: 'search',
		placeholder: 'Search...',
		clearable: true,
	},
}

export const Small: Story = {
	args: {
		icon: SearchIcon,
		placeholder: 'Filter options...',
		size: 'small',
	},
}

export const Disabled: Story = {
	args: {
		icon: UserIcon,
		placeholder: 'Disabled input',
		disabled: true,
	},
}

export const Readonly: Story = {
	render: () => ({
		components: { Input },
		setup() {
			const value = ref('This is readonly')
			return { value, UserIcon }
		},
		template: `
			<Input
				v-model="value"
				:icon="UserIcon"
				placeholder="Readonly input"
				readonly
			/>
		`,
	}),
}

export const Error: Story = {
	render: () => ({
		components: { Input },
		setup() {
			const value = ref('invalid-url')
			return { value, LinkIcon }
		},
		template: `
			<Input
				v-model="value"
				:icon="LinkIcon"
				type="url"
				placeholder="Enter URL..."
				error
			/>
		`,
	}),
}

export const AllSizes: StoryObj = {
	render: () => ({
		components: { Input },
		setup() {
			return { SearchIcon }
		},
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem;">
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Standard (36px)</p>
					<Input
						:icon="SearchIcon"
						placeholder="Standard size..."
						size="standard"
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Small (32px)</p>
					<Input
						:icon="SearchIcon"
						placeholder="Small size..."
						size="small"
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Medium (40px)</p>
					<Input
						:icon="SearchIcon"
						placeholder="Medium size..."
						size="medium"
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Large (48px)</p>
					<Input
						:icon="SearchIcon"
						placeholder="Large size..."
						size="large"
					/>
				</div>
			</div>
		`,
	}),
}

export const AllStates: StoryObj = {
	render: () => ({
		components: { Input },
		setup() {
			const normalValue = ref('')
			const filledValue = ref('With content')
			const errorValue = ref('invalid')
			const readonlyValue = ref('Readonly content')
			return { normalValue, filledValue, errorValue, readonlyValue, SearchIcon, LinkIcon }
		},
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem; max-width: 300px;">
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Default</p>
					<Input
						v-model="normalValue"
						:icon="SearchIcon"
						placeholder="Enter text..."
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">With Value + Clearable</p>
					<Input
						v-model="filledValue"
						:icon="SearchIcon"
						placeholder="Enter text..."
						clearable
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Error State</p>
					<Input
						v-model="errorValue"
						:icon="LinkIcon"
						placeholder="Enter URL..."
						error
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Disabled</p>
					<Input
						:icon="SearchIcon"
						placeholder="Disabled..."
						disabled
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Readonly</p>
					<Input
						v-model="readonlyValue"
						:icon="SearchIcon"
						placeholder="Readonly..."
						readonly
					/>
				</div>
			</div>
		`,
	}),
}
