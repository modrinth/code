import type { Meta, StoryObj } from '@storybook/vue3-vite'

import Accordion from '../../components/base/Accordion.vue'

const meta = {
	title: 'Base/Accordion',
	component: Accordion,
	render: (args) => ({
		components: { Accordion },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<Accordion v-bind="args">
				<template #title>Click to expand</template>
				<p>This is the accordion content that shows when expanded.</p>
			</Accordion>
		`,
	}),
} satisfies Meta<typeof Accordion>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const OpenByDefault: Story = {
	args: {
		openByDefault: true,
	},
}

export const ForceOpen: Story = {
	args: {
		forceOpen: true,
	},
}

export const WithCustomButtonClass: StoryObj = {
	render: () => ({
		components: { Accordion },
		template: /*html*/ `
			<Accordion
				open-by-default
				button-class="text-primary m-0 p-0 border-none bg-transparent active:scale-95"
			>
				<template #title>3 servers</template>
				<div class="mt-2 flex flex-wrap gap-2">
					<span class="px-2 py-1 bg-bg-raised rounded text-sm">server-001</span>
					<span class="px-2 py-1 bg-bg-raised rounded text-sm">server-002</span>
					<span class="px-2 py-1 bg-bg-raised rounded text-sm">server-003</span>
				</div>
			</Accordion>
		`,
	}),
}

export const WithSummarySlot: StoryObj = {
	render: () => ({
		components: { Accordion },
		template: /*html*/ `
			<Accordion>
				<template #title>
					<span class="text-lg font-bold text-contrast">Advanced Settings</span>
				</template>
				<template #summary>
					<span class="text-sm text-secondary">Configure advanced options for your project</span>
				</template>
				<div class="mt-4 flex flex-col gap-2">
					<label class="flex items-center gap-2">
						<input type="checkbox" />
						<span>Enable debug mode</span>
					</label>
					<label class="flex items-center gap-2">
						<input type="checkbox" />
						<span>Show verbose output</span>
					</label>
				</div>
			</Accordion>
		`,
	}),
}

export const WithContentClass: StoryObj = {
	render: () => ({
		components: { Accordion },
		template: /*html*/ `
			<Accordion open-by-default content-class="p-4 bg-bg-raised rounded-lg mt-2">
				<template #title>
					<span class="text-primary font-semibold">Styled Content Area</span>
				</template>
				<p>This content has custom padding and background styling applied via contentClass.</p>
			</Accordion>
		`,
	}),
}

export const MultipleAccordions: StoryObj = {
	render: () => ({
		components: { Accordion },
		template: /*html*/ `
			<div class="flex flex-col gap-4">
				<Accordion>
					<template #title>
						<span class="text-primary font-semibold">Section 1: Getting Started</span>
					</template>
					<div class="mt-2 text-secondary">
						<p>Welcome! This section covers the basics of getting started with the application.</p>
					</div>
				</Accordion>
				<Accordion>
					<template #title>
						<span class="text-primary font-semibold">Section 2: Configuration</span>
					</template>
					<div class="mt-2 text-secondary">
						<p>Learn how to configure your settings and customize the experience.</p>
					</div>
				</Accordion>
				<Accordion open-by-default>
					<template #title>
						<span class="text-primary font-semibold">Section 3: FAQ (Open by default)</span>
					</template>
					<div class="mt-2 text-secondary">
						<p>Frequently asked questions and their answers.</p>
					</div>
				</Accordion>
			</div>
		`,
	}),
}

export const NestedContent: StoryObj = {
	render: () => ({
		components: { Accordion },
		template: /*html*/ `
			<Accordion open-by-default>
				<template #title>
					<span class="text-primary font-semibold">Project Dependencies</span>
				</template>
				<div class="mt-2 flex flex-col gap-3">
					<div class="flex items-center justify-between p-2 bg-bg-raised rounded">
						<span>vue</span>
						<span class="text-sm text-secondary">^3.5.0</span>
					</div>
					<div class="flex items-center justify-between p-2 bg-bg-raised rounded">
						<span>typescript</span>
						<span class="text-sm text-secondary">^5.0.0</span>
					</div>
					<div class="flex items-center justify-between p-2 bg-bg-raised rounded">
						<span>vite</span>
						<span class="text-sm text-secondary">^6.0.0</span>
					</div>
				</div>
			</Accordion>
		`,
	}),
}

export const AllStates: StoryObj = {
	render: () => ({
		components: { Accordion },
		template: /*html*/ `
			<div class="flex flex-col gap-6">
				<div>
					<h3 class="text-sm font-semibold text-secondary mb-2">Default (collapsed)</h3>
					<Accordion>
						<template #title><span class="text-primary">Click to expand</span></template>
						<p class="mt-2">Hidden content revealed on click.</p>
					</Accordion>
				</div>
				<div>
					<h3 class="text-sm font-semibold text-secondary mb-2">Open by default</h3>
					<Accordion open-by-default>
						<template #title><span class="text-primary">Already expanded</span></template>
						<p class="mt-2">This content is visible by default.</p>
					</Accordion>
				</div>
				<div>
					<h3 class="text-sm font-semibold text-secondary mb-2">Force open (no toggle)</h3>
					<Accordion force-open>
						<template #title><span class="text-primary">Cannot be collapsed</span></template>
						<p class="mt-2">This accordion is always open and cannot be toggled.</p>
					</Accordion>
				</div>
			</div>
		`,
	}),
}
