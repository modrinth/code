import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Page from '../../components/base/Page.vue'

const meta = {
	title: 'Base/Page',
	component: Page,
} satisfies Meta<typeof Page>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => ({
		components: { Page },
		template: `
			<Page>
				<div class="p-4 bg-bg-raised rounded-lg">
					<h1 class="text-2xl font-bold mb-4">Page Content</h1>
					<p>This is the main content area of the page.</p>
				</div>
			</Page>
		`,
	}),
}

export const WithSidebar: Story = {
	render: () => ({
		components: { Page },
		template: `
			<Page>
				<template #sidebar>
					<div class="p-4 bg-bg-raised rounded-lg">
						<h2 class="text-xl font-bold mb-2">Sidebar</h2>
						<ul class="space-y-2">
							<li>Link 1</li>
							<li>Link 2</li>
							<li>Link 3</li>
						</ul>
					</div>
				</template>
				<div class="p-4 bg-bg-raised rounded-lg">
					<h1 class="text-2xl font-bold mb-4">Page Content</h1>
					<p>This is the main content area with a sidebar.</p>
				</div>
			</Page>
		`,
	}),
}

export const WithHeaderAndFooter: Story = {
	render: () => ({
		components: { Page },
		template: `
			<Page>
				<template #header>
					<div class="p-4 bg-brand-highlight rounded-lg mb-4">
						<h1 class="text-2xl font-bold">Page Header</h1>
					</div>
				</template>
				<template #sidebar>
					<div class="p-4 bg-bg-raised rounded-lg">
						<h2 class="text-xl font-bold mb-2">Sidebar</h2>
					</div>
				</template>
				<div class="p-4 bg-bg-raised rounded-lg">
					<h2 class="text-xl font-bold mb-4">Main Content</h2>
					<p>Content with header, sidebar, and footer.</p>
				</div>
				<template #footer>
					<div class="p-4 bg-bg-raised rounded-lg mt-4">
						<p class="text-secondary">Footer content</p>
					</div>
				</template>
			</Page>
		`,
	}),
}
