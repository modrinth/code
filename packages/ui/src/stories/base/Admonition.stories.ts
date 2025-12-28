import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Admonition from '../../components/base/Admonition.vue'

const meta = {
	title: 'Base/Admonition',
	component: Admonition,
} satisfies Meta<typeof Admonition>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		body: 'This is an informational message.',
	},
}

export const AllTypes: Story = {
	render: () => ({
		components: { Admonition },
		template: /*html*/ `
			<div style="display: flex; flex-direction: column; gap: 1rem;">
				<Admonition type="info" header="Info" body="This is an informational message." />
				<Admonition type="warning" header="Warning" body="This is a warning message." />
				<Admonition type="critical" header="Critical" body="This is a critical message." />
			</div>
		`,
	}),
}

export const WithHeader: Story = {
	args: {
		type: 'warning',
		header: 'Important Notice',
		body: 'Please read this carefully before proceeding.',
	},
}
