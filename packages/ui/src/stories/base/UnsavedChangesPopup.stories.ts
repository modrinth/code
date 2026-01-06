import type { Meta, StoryObj } from '@storybook/vue3-vite'

import UnsavedChangesPopup from '../../components/base/UnsavedChangesPopup.vue'

const meta = {
	title: 'Base/UnsavedChangesPopup',
	// @ts-ignore
	component: UnsavedChangesPopup,
} satisfies Meta<typeof UnsavedChangesPopup>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { UnsavedChangesPopup },
		template: `
			<div class="relative h-32">
				<UnsavedChangesPopup
					:original="{ name: 'Original Name' }"
					:modified="{ name: 'Modified Name' }"
					@save="() => console.log('Save clicked')"
					@reset="() => console.log('Reset clicked')"
				/>
			</div>
		`,
	}),
}

export const Saving: StoryObj = {
	render: () => ({
		components: { UnsavedChangesPopup },
		template: `
			<div class="relative h-32">
				<UnsavedChangesPopup
					:original="{ name: 'Original' }"
					:modified="{ name: 'Changed' }"
					:saving="true"
					@save="() => console.log('Save clicked')"
					@reset="() => console.log('Reset clicked')"
				/>
			</div>
		`,
	}),
}

export const NoResetButton: StoryObj = {
	render: () => ({
		components: { UnsavedChangesPopup },
		template: `
			<div class="relative h-32">
				<UnsavedChangesPopup
					:original="{ name: 'Original' }"
					:modified="{ name: 'Changed' }"
					:canReset="false"
					@save="() => console.log('Save clicked')"
				/>
			</div>
		`,
	}),
}

export const Hidden: StoryObj = {
	render: () => ({
		components: { UnsavedChangesPopup },
		template: `
			<div class="relative h-32">
				<p class="text-secondary mb-4">No changes detected - popup is hidden</p>
				<UnsavedChangesPopup
					:original="{ name: 'Same Value' }"
					:modified="{ name: 'Same Value' }"
					@save="() => console.log('Save clicked')"
					@reset="() => console.log('Reset clicked')"
				/>
			</div>
		`,
	}),
}
