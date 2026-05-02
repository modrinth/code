import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { onMounted, ref } from 'vue'

import LoadingBar from '../../components/base/LoadingBar.vue'
import { createLoadingStateCore } from '../../composables/use-loading-state-core'
import { provideLoadingState } from '../../providers/loading-state'

const meta = {
	title: 'Base/LoadingBar',
	component: LoadingBar,
} satisfies Meta<typeof LoadingBar>

export default meta
type Story = StoryObj<typeof meta>

export const Idle: Story = {
	render: () => ({
		components: { LoadingBar },
		setup() {
			provideLoadingState(createLoadingStateCore())
			return {}
		},
		template: `
			<div class="relative h-32 w-full">
				<LoadingBar />
				<p class="text-secondary">Loading bar is idle (no active tokens).</p>
			</div>
		`,
	}),
}

export const SinglePending: Story = {
	render: () => ({
		components: { LoadingBar },
		setup() {
			const core = createLoadingStateCore()
			provideLoadingState(core)
			onMounted(() => {
				core.begin()
			})
			return {}
		},
		template: `
			<div class="relative h-32 w-full">
				<LoadingBar />
				<p class="text-secondary">One token registered — bar fills to 100% over 1s.</p>
			</div>
		`,
	}),
}

export const StackedPending: Story = {
	render: () => ({
		components: { LoadingBar },
		setup() {
			const core = createLoadingStateCore()
			provideLoadingState(core)
			const tokens: symbol[] = []
			onMounted(() => {
				tokens.push(core.begin())
				tokens.push(core.begin())
				setTimeout(() => core.end(tokens[0]!), 1500)
				setTimeout(() => core.end(tokens[1]!), 3000)
			})
			return {}
		},
		template: `
			<div class="relative h-32 w-full">
				<LoadingBar />
				<p class="text-secondary">Two tokens. First releases at 1.5s, second at 3s — bar stays visible until both end.</p>
			</div>
		`,
	}),
}

export const ManualRefresh: Story = {
	render: () => ({
		components: { LoadingBar },
		setup() {
			const core = createLoadingStateCore()
			provideLoadingState(core)
			const last = ref<string>('idle')
			function trigger() {
				core.beginManual(800)
				last.value = `beginManual(800) at ${new Date().toLocaleTimeString()}`
			}
			return { trigger, last }
		},
		template: `
			<div class="relative h-32 w-full">
				<LoadingBar />
				<button class="rounded bg-button-bg px-3 py-2 text-contrast" @click="trigger">Manual refresh</button>
				<p class="text-secondary mt-2">{{ last }}</p>
			</div>
		`,
	}),
}
