import { KeyIcon, LinkIcon, MailIcon, SearchIcon, UserIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import StyledInput from '../../components/base/StyledInput.vue'

const meta = {
	title: 'Base/StyledInput',
	component: StyledInput,
	argTypes: {
		size: {
			control: 'select',
			options: ['standard', 'small'],
		},
		type: {
			control: 'select',
			options: ['text', 'email', 'password', 'number', 'url', 'search', 'date', 'datetime-local'],
		},
	},
} satisfies Meta<typeof StyledInput>

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

export const Clearable: Story = {
	render: () => ({
		components: { StyledInput },
		setup() {
			const value = ref('Some text to clear')
			return { value, SearchIcon }
		},
		template: `
			<StyledInput
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
		components: { StyledInput },
		setup() {
			const value = ref('Search query')
			return { value, SearchIcon }
		},
		template: `
			<StyledInput
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

export const Date: Story = {
	args: {
		type: 'date',
	},
}

export const DatetimeLocal: Story = {
	args: {
		type: 'datetime-local',
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
		components: { StyledInput },
		setup() {
			const value = ref('This is readonly')
			return { value, UserIcon }
		},
		template: `
			<StyledInput
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
		components: { StyledInput },
		setup() {
			const value = ref('invalid-url')
			return { value, LinkIcon }
		},
		template: `
			<StyledInput
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
		components: { StyledInput },
		setup() {
			return { SearchIcon }
		},
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem;">
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Standard (36px)</p>
					<StyledInput
						:icon="SearchIcon"
						placeholder="Standard size..."
						size="standard"
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Small (32px)</p>
					<StyledInput
						:icon="SearchIcon"
						placeholder="Small size..."
						size="small"
					/>
				</div>
			</div>
		`,
	}),
}

export const AllStates: StoryObj = {
	render: () => ({
		components: { StyledInput },
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
					<StyledInput
						v-model="normalValue"
						:icon="SearchIcon"
						placeholder="Enter text..."
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">With Value + Clearable</p>
					<StyledInput
						v-model="filledValue"
						:icon="SearchIcon"
						placeholder="Enter text..."
						clearable
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Error State</p>
					<StyledInput
						v-model="errorValue"
						:icon="LinkIcon"
						placeholder="Enter URL..."
						error
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Disabled</p>
					<StyledInput
						:icon="SearchIcon"
						placeholder="Disabled..."
						disabled
					/>
				</div>
				<div>
					<p style="margin-bottom: 0.5rem; font-weight: 600;">Readonly</p>
					<StyledInput
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
