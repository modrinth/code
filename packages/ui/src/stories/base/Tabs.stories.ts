import { ChartIcon, DownloadIcon, UsersIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import Tabs from '../../components/base/Tabs.vue'

const meta = {
	title: 'Base/Tabs',
	component: Tabs,
} satisfies Meta<typeof Tabs>

export default meta

export const Default: StoryObj = {
	render: () => ({
		components: { Tabs },
		setup() {
			const value = ref('area')
			const tabs = [
				{ value: 'line', label: 'Line' },
				{ value: 'area', label: 'Area' },
				{ value: 'bar', label: 'Bar' },
			]

			return { value, tabs }
		},
		template: /* html */ `
			<Tabs v-model:value="value" :tabs="tabs" />
		`,
	}),
}

export const WithIcons: StoryObj = {
	render: () => ({
		components: { Tabs },
		setup() {
			const value = ref('downloads')
			const tabs = [
				{ value: 'overview', label: 'Overview', icon: ChartIcon },
				{ value: 'downloads', label: 'Downloads', icon: DownloadIcon },
				{ value: 'users', label: 'Users', icon: UsersIcon },
			]

			return { value, tabs }
		},
		template: /* html */ `
			<Tabs v-model:value="value" :tabs="tabs" />
		`,
	}),
}
