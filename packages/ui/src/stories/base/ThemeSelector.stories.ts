import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import ThemeSelector from '../../components/settings/ThemeSelector.vue'

const meta = {
	title: 'Settings/ThemeSelector',
	// @ts-ignore - error comes from generically typed component
	component: ThemeSelector,
} satisfies Meta<typeof ThemeSelector>

export default meta

export const Interactive: StoryObj = {
	render: () => ({
		components: { ThemeSelector },
		setup() {
			const currentTheme = ref('dark')
			const themeOptions = ['system', 'light', 'dark', 'oled', 'retro']
			return { currentTheme, themeOptions }
		},
		template: `
			<ThemeSelector
				aria-label="Color theme"
				v-model="currentTheme"
				:theme-options="themeOptions"
				system-theme-color="dark"
			/>
		`,
	}),
}
