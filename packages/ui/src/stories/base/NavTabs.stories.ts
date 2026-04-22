import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { getCurrentInstance, ref } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'

import NavTabs from '../../components/base/NavTabs.vue'

const router = createRouter({
	history: createMemoryHistory(),
	routes: [{ path: '/', component: { template: '<div />' } }],
})

const meta = {
	title: 'Base/NavTabs',
	component: NavTabs,
	decorators: [
		(story) => {
			return {
				components: { story },
				setup() {
					const app = getCurrentInstance()?.appContext.app
					if (app && !app.config.globalProperties.$router) {
						app.use(router)
					}
				},
				template: '<story />',
			}
		},
	],
} satisfies Meta<typeof NavTabs>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	args: {
		mode: 'local',
		activeIndex: 0,
		links: [
			{ label: 'Overview', href: '' },
			{ label: 'Gallery', href: 'gallery' },
			{ label: 'Changelog', href: 'changelog' },
			{ label: 'Versions', href: 'versions' },
		],
	},
}

export const LocalMode: StoryObj = {
	render: () => ({
		components: { NavTabs },
		setup() {
			const activeIndex = ref(0)
			const links = [
				{ label: 'Overview', href: '' },
				{ label: 'Gallery', href: 'gallery' },
				{ label: 'Changelog', href: 'changelog' },
				{ label: 'Versions', href: 'versions' },
			]
			function onTabClick(index: number) {
				activeIndex.value = index
			}
			return { activeIndex, links, onTabClick }
		},
		template: /* html */ `
			<NavTabs
				mode="local"
				:active-index="activeIndex"
				:links="links"
				@tab-click="onTabClick"
			/>
		`,
	}),
}

export const TwoTabs: StoryObj = {
	render: () => ({
		components: { NavTabs },
		setup() {
			const activeIndex = ref(0)
			const links = [
				{ label: 'Details', href: '' },
				{ label: 'Settings', href: 'settings' },
			]
			function onTabClick(index: number) {
				activeIndex.value = index
			}
			return { activeIndex, links, onTabClick }
		},
		template: /* html */ `
			<NavTabs
				mode="local"
				:active-index="activeIndex"
				:links="links"
				@tab-click="onTabClick"
			/>
		`,
	}),
}
