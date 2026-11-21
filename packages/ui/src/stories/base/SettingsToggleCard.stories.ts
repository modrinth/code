import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import SettingsToggleCard from '../../components/base/SettingsToggleCard.vue'

const meta = {
	title: 'Base/SettingsToggleCard',
	component: SettingsToggleCard,
} satisfies Meta<typeof SettingsToggleCard>

export default meta

export const WithGuidance: StoryObj = {
	render: () => ({
		components: { SettingsToggleCard },
		setup: () => ({ enabled: ref(false) }),
		template: /*html*/ `
			<SettingsToggleCard
				v-model="enabled"
				title="Show status"
				description="Display project status on the project page."
			>
				<template #after-summary>
					<details class="p-4">
						<summary>Where does the status appear?</summary>
						<p>Visitors see it below the project title.</p>
					</details>
				</template>
				<template #expanded>
					<label>Custom status <input type="text" /></label>
				</template>
			</SettingsToggleCard>
		`,
	}),
}
