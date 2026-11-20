import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import PageHeaderBackButton from '../../components/base/page-header/page-header-back-button.vue'

const meta = {
	title: 'Base/PageHeaderBackButton',
	component: PageHeaderBackButton,
} satisfies Meta<typeof PageHeaderBackButton>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { PageHeaderBackButton },
		setup() {
			const hidden = ref(false)
			return { hidden }
		},
		template: /* html */ `
			<div class="flex flex-col gap-4">
				<label class="flex items-center gap-2">
					<input v-model="hidden" type="checkbox" />
					Hidden
				</label>
				<div class="flex items-center gap-4">
					<PageHeaderBackButton label="Back to instance" :hidden="hidden" />
					<div class="size-16 rounded-xl bg-surface-4" />
				</div>
			</div>
		`,
	}),
}
