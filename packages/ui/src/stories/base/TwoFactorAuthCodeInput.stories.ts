import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import TwoFactorAuthCodeInput from '../../components/base/TwoFactorAuthCodeInput.vue'

const meta = {
	title: 'Base/TwoFactorAuthCodeInput',
	component: TwoFactorAuthCodeInput,
} satisfies Meta<typeof TwoFactorAuthCodeInput>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { TwoFactorAuthCodeInput },
		setup() {
			const code = ref('')
			return { code }
		},
		template: `
			<TwoFactorAuthCodeInput v-model="code" />
		`,
	}),
}

export const Filled: Story = {
	render: () => ({
		components: { TwoFactorAuthCodeInput },
		setup() {
			const code = ref('123456')
			return { code }
		},
		template: `
			<TwoFactorAuthCodeInput v-model="code" />
		`,
	}),
}

export const Disabled: Story = {
	render: () => ({
		components: { TwoFactorAuthCodeInput },
		setup() {
			const code = ref('123456')
			return { code }
		},
		template: `
			<TwoFactorAuthCodeInput v-model="code" disabled />
		`,
	}),
}
