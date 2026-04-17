import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { onMounted, ref } from 'vue'

import LoadingBar from '../../components/base/LoadingBar.vue'
import ReadyTransition from '../../components/base/ReadyTransition.vue'
import { createLoadingStateCore } from '../../composables/use-loading-state-core'
import { provideLoadingState } from '../../providers/loading-state'

const meta = {
	title: 'Base/ReadyTransition',
	component: ReadyTransition,
} satisfies Meta<typeof ReadyTransition>

export default meta
type Story = StoryObj<typeof meta>

export const Idle: Story = {
	render: () => ({
		components: { ReadyTransition, LoadingBar },
		setup() {
			provideLoadingState(createLoadingStateCore())
			return { pending: ref(false) }
		},
		template: `
			<div class="relative">
				<LoadingBar />
				<ReadyTransition :pending="pending">
					<div class="rounded bg-bg-raised p-4 text-contrast">Slot content (already ready).</div>
				</ReadyTransition>
			</div>
		`,
	}),
}

export const PendingThenReady: Story = {
	render: () => ({
		components: { ReadyTransition, LoadingBar },
		setup() {
			provideLoadingState(createLoadingStateCore())
			const pending = ref(true)
			onMounted(() => {
				setTimeout(() => (pending.value = false), 2000)
			})
			return { pending }
		},
		template: `
			<div class="relative">
				<LoadingBar />
				<p class="text-secondary mb-4">Pending for 2s, then content fades in. Bar runs at top.</p>
				<ReadyTransition :pending="pending">
					<div class="rounded bg-bg-raised p-4 text-contrast">Slot content revealed.</div>
				</ReadyTransition>
			</div>
		`,
	}),
}

export const StackedTwoTransitions: Story = {
	render: () => ({
		components: { ReadyTransition, LoadingBar },
		setup() {
			provideLoadingState(createLoadingStateCore())
			const a = ref(true)
			const b = ref(true)
			onMounted(() => {
				setTimeout(() => (a.value = false), 1500)
				setTimeout(() => (b.value = false), 3000)
			})
			return { a, b }
		},
		template: `
			<div class="relative grid gap-4">
				<LoadingBar />
				<ReadyTransition :pending="a">
					<div class="rounded bg-bg-raised p-4 text-contrast">Panel A (ready at 1.5s).</div>
				</ReadyTransition>
				<ReadyTransition :pending="b">
					<div class="rounded bg-bg-raised p-4 text-contrast">Panel B (ready at 3s).</div>
				</ReadyTransition>
				<p class="text-secondary">Bar stays visible until BOTH panels resolve.</p>
			</div>
		`,
	}),
}

export const Silent: Story = {
	render: () => ({
		components: { ReadyTransition, LoadingBar },
		setup() {
			provideLoadingState(createLoadingStateCore())
			const pending = ref(true)
			onMounted(() => {
				setTimeout(() => (pending.value = false), 1500)
			})
			return { pending }
		},
		template: `
			<div class="relative">
				<LoadingBar />
				<p class="text-secondary mb-4">silent=true — fades locally but does NOT raise the loading bar.</p>
				<ReadyTransition :pending="pending" silent>
					<div class="rounded bg-bg-raised p-4 text-contrast">Silent slot content.</div>
				</ReadyTransition>
			</div>
		`,
	}),
}
