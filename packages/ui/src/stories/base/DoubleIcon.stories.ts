import { DownloadIcon, HeartIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import DoubleIcon from '../../components/base/DoubleIcon.vue'

const meta = {
	title: 'Base/DoubleIcon',
	component: DoubleIcon,
	render: (args) => ({
		components: { DoubleIcon, DownloadIcon, HeartIcon },
		setup() {
			return { args }
		},
		template: /*html*/ `
			<DoubleIcon v-bind="args">
				<template #primary><DownloadIcon style="width: 2rem; height: 2rem;" /></template>
				<template #secondary><HeartIcon /></template>
			</DoubleIcon>
		`,
	}),
} satisfies Meta<typeof DoubleIcon>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
