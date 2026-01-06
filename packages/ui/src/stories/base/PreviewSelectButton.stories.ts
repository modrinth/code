import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import PreviewSelectButton from '../../components/base/PreviewSelectButton.vue'

const meta = {
	title: 'Base/PreviewSelectButton',
	component: PreviewSelectButton,
} satisfies Meta<typeof PreviewSelectButton>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { PreviewSelectButton },
		template: `
			<PreviewSelectButton :checked="false">
				<template #preview>
					<div class="w-16 h-16 bg-brand-highlight rounded-lg" />
				</template>
				Option Label
			</PreviewSelectButton>
		`,
	}),
}

export const AllStates: StoryObj = {
	render: () => ({
		components: { PreviewSelectButton },
		template: `
			<div class="flex gap-4">
				<PreviewSelectButton :checked="false">
					<template #preview>
						<div class="w-16 h-16 bg-bg-raised rounded-lg flex items-center justify-center border border-divider">
							<span class="text-secondary">A</span>
						</div>
					</template>
					Unchecked
				</PreviewSelectButton>
				<PreviewSelectButton :checked="true">
					<template #preview>
						<div class="w-16 h-16 bg-brand-highlight rounded-lg flex items-center justify-center border border-brand">
							<span class="text-brand">B</span>
						</div>
					</template>
					Checked
				</PreviewSelectButton>
			</div>
		`,
	}),
}

export const InteractiveSelection: StoryObj = {
	render: () => ({
		components: { PreviewSelectButton },
		setup() {
			const selected = ref('dark')
			return { selected }
		},
		template: `
			<div>
				<p class="text-sm text-secondary mb-4">Selected: {{ selected }}</p>
				<div class="grid grid-cols-3 gap-4">
					<PreviewSelectButton
						:checked="selected === 'light'"
						@click="selected = 'light'"
					>
						<template #preview>
							<div class="w-16 h-16 bg-white rounded-lg border border-divider" />
						</template>
						Light
					</PreviewSelectButton>
					<PreviewSelectButton
						:checked="selected === 'dark'"
						@click="selected = 'dark'"
					>
						<template #preview>
							<div class="w-16 h-16 bg-gray-900 rounded-lg border border-divider" />
						</template>
						Dark
					</PreviewSelectButton>
					<PreviewSelectButton
						:checked="selected === 'oled'"
						@click="selected = 'oled'"
					>
						<template #preview>
							<div class="w-16 h-16 bg-black rounded-lg border border-divider" />
						</template>
						OLED
					</PreviewSelectButton>
				</div>
			</div>
		`,
	}),
}

export const ColorSelection: StoryObj = {
	render: () => ({
		components: { PreviewSelectButton },
		setup() {
			const selected = ref('brand')
			return { selected }
		},
		template: `
			<div>
				<p class="text-sm text-secondary mb-4">Accent color: {{ selected }}</p>
				<div class="grid grid-cols-4 gap-3">
					<PreviewSelectButton
						v-for="color in ['brand', 'red', 'orange', 'green', 'blue', 'purple']"
						:key="color"
						:checked="selected === color"
						@click="selected = color"
					>
						<template #preview>
							<div
								class="w-12 h-12 rounded-lg"
								:class="{
									'bg-brand': color === 'brand',
									'bg-red': color === 'red',
									'bg-orange': color === 'orange',
									'bg-green': color === 'green',
									'bg-blue': color === 'blue',
									'bg-purple': color === 'purple',
								}"
							/>
						</template>
						{{ color.charAt(0).toUpperCase() + color.slice(1) }}
					</PreviewSelectButton>
				</div>
			</div>
		`,
	}),
}
