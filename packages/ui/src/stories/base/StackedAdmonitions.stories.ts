import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import Admonition from '../../components/base/Admonition.vue'
import ButtonStyled from '../../components/base/ButtonStyled.vue'
import ProgressBar from '../../components/base/ProgressBar.vue'
import StackedAdmonitionsRaw, {
	type StackedAdmonitionItem,
} from '../../components/base/StackedAdmonitions.vue'

// The generic type signature of StackedAdmonitions breaks Storybook's Meta
// inference and Vue's components record type. Cast to `any` for story wiring;
// runtime behavior is unchanged.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const StackedAdmonitions = StackedAdmonitionsRaw as any

interface DemoItem extends StackedAdmonitionItem {
	header: string
	body: string
}

const meta: Meta = {
	title: 'Base/StackedAdmonitions',
	component: StackedAdmonitions,
}

export default meta
type Story = StoryObj

const initialItems: DemoItem[] = [
	{
		id: 'backup-failed',
		type: 'critical',
		header: 'Backup failed',
		body: 'Something went wrong while creating your backup. Try again or contact support.',
	},
	{
		id: 'storage-nearly-full',
		type: 'warning',
		header: 'Storage nearly full',
		body: 'Your server is using 92% of available storage.',
	},
	{
		id: 'scheduled-maintenance',
		type: 'info',
		header: 'Scheduled maintenance',
		body: 'Routine maintenance will begin in 30 minutes.',
	},
]

export const Empty: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition },
		template: /* html */ `
			<div style="min-height: 4rem;">
				<StackedAdmonitions :items="[]">
					<template #item="{ item }">
						<Admonition :type="item.type" :header="item.header" :body="item.body" />
					</template>
				</StackedAdmonitions>
				<p style="color: var(--color-secondary); margin-top: 1rem;">
					Nothing should render above this line.
				</p>
			</div>
		`,
	}),
}

export const SingleItem: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition },
		setup() {
			const items = ref<DemoItem[]>([initialItems[0]])
			return { items }
		},
		template: /* html */ `
			<StackedAdmonitions :items="items">
				<template #item="{ item }">
					<Admonition :type="item.type" :header="item.header" :body="item.body" />
				</template>
			</StackedAdmonitions>
		`,
	}),
}

export const TwoItems: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition },
		setup() {
			const items = ref<DemoItem[]>(initialItems.slice(0, 2))
			return { items }
		},
		template: /* html */ `
			<StackedAdmonitions :items="items">
				<template #item="{ item }">
					<Admonition :type="item.type" :header="item.header" :body="item.body" />
				</template>
			</StackedAdmonitions>
		`,
	}),
}

export const FiveItems: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition },
		setup() {
			const items = ref<DemoItem[]>([
				...initialItems,
				{
					id: 'update-available',
					type: 'success',
					header: 'Update available',
					body: 'A new version of your server software is ready to install.',
				},
				{
					id: 'memory-pressure',
					type: 'warning',
					header: 'High memory usage',
					body: 'Your server is using 89% of allocated memory.',
				},
			])
			return { items }
		},
		template: /* html */ `
			<StackedAdmonitions :items="items">
				<template #item="{ item }">
					<Admonition :type="item.type" :header="item.header" :body="item.body" />
				</template>
			</StackedAdmonitions>
		`,
	}),
}

export const MixedTypes: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition },
		setup() {
			const items = ref<DemoItem[]>([
				{
					id: 'critical',
					type: 'critical',
					header: 'Critical admonition',
					body: 'Red background placeholder when not front.',
				},
				{
					id: 'warning',
					type: 'warning',
					header: 'Warning admonition',
					body: 'Orange background placeholder when not front.',
				},
				{
					id: 'info',
					type: 'info',
					header: 'Info admonition',
					body: 'Blue background placeholder when not front.',
				},
				{
					id: 'success',
					type: 'success',
					header: 'Success admonition',
					body: 'Green background placeholder when not front.',
				},
			])
			return { items }
		},
		template: /* html */ `
			<StackedAdmonitions :items="items">
				<template #item="{ item }">
					<Admonition :type="item.type" :header="item.header" :body="item.body" />
				</template>
			</StackedAdmonitions>
		`,
	}),
}

export const ForceExpanded: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition },
		setup() {
			const items = ref<DemoItem[]>(initialItems)
			return { items }
		},
		template: /* html */ `
			<StackedAdmonitions :items="items" :expanded="true">
				<template #item="{ item }">
					<Admonition :type="item.type" :header="item.header" :body="item.body" />
				</template>
			</StackedAdmonitions>
		`,
	}),
}

export const DismissIndividual: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition },
		setup() {
			const items = ref<DemoItem[]>([...initialItems])
			function dismiss(id: string) {
				items.value = items.value.filter((i) => i.id !== id)
			}
			function reset() {
				items.value = [...initialItems]
			}
			return { items, dismiss, reset }
		},
		template: /* html */ `
			<div>
				<button
					type="button"
					style="margin-bottom: 0.75rem; padding: 0.25rem 0.75rem; border-radius: 0.5rem; background: var(--color-button-bg); color: var(--color-contrast);"
					@click="reset"
				>
					Reset
				</button>
				<StackedAdmonitions :items="items">
					<template #item="{ item }">
						<Admonition
							:type="item.type"
							:header="item.header"
							:body="item.body"
							dismissible
							@dismiss="dismiss(item.id)"
						/>
					</template>
				</StackedAdmonitions>
			</div>
		`,
	}),
}

export const DismissAll: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition },
		setup() {
			const items = ref<DemoItem[]>([...initialItems])
			function dismiss(id: string) {
				items.value = items.value.filter((i) => i.id !== id)
			}
			function dismissAll() {
				items.value = []
			}
			function reset() {
				items.value = [...initialItems]
			}
			return { items, dismiss, dismissAll, reset }
		},
		template: /* html */ `
			<div>
				<button
					type="button"
					style="margin-bottom: 0.75rem; padding: 0.25rem 0.75rem; border-radius: 0.5rem; background: var(--color-button-bg); color: var(--color-contrast);"
					@click="reset"
				>
					Reset
				</button>
				<StackedAdmonitions :items="items" @dismiss-all="dismissAll">
					<template #item="{ item }">
						<Admonition
							:type="item.type"
							:header="item.header"
							:body="item.body"
							dismissible
							@dismiss="dismiss(item.id)"
						/>
					</template>
				</StackedAdmonitions>
			</div>
		`,
	}),
}

interface RichItem extends StackedAdmonitionItem {
	header: string
	body: string
	progress?: number
	canRetry?: boolean
	canCancel?: boolean
}

export const RichContent: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition, ButtonStyled, ProgressBar },
		setup() {
			const items = ref<RichItem[]>([
				{
					id: 'upload-1',
					type: 'info',
					header: 'Uploading files (2/5)',
					body: '128 KB / 1.2 MB (45%)',
					progress: 0.45,
					canCancel: true,
				},
				{
					id: 'extract-1',
					type: 'critical',
					header: 'Extraction failed',
					body: 'Something went wrong while extracting the archive.',
					canRetry: true,
				},
				{
					id: 'install-1',
					type: 'success',
					header: 'Installation complete',
					body: 'All files have been installed successfully.',
				},
			])
			function dismiss(id: string) {
				items.value = items.value.filter((i) => i.id !== id)
			}
			function dismissAll() {
				items.value = []
			}
			return { items, dismiss, dismissAll }
		},
		template: /* html */ `
			<StackedAdmonitions :items="items" @dismiss-all="dismissAll">
				<template #item="{ item }">
					<Admonition :type="item.type" :header="item.header">
						{{ item.body }}
						<template #top-right-actions>
							<ButtonStyled v-if="item.canCancel" type="outlined" color="blue">
								<button class="!border" @click="dismiss(item.id)">Cancel</button>
							</ButtonStyled>
							<ButtonStyled v-if="item.canRetry" color="red">
								<button @click="dismiss(item.id)">Retry</button>
							</ButtonStyled>
							<ButtonStyled circular type="transparent" hover-color-fill="background" :color="item.type === 'critical' ? 'red' : item.type === 'success' ? 'green' : 'blue'">
								<button @click="dismiss(item.id)">✕</button>
							</ButtonStyled>
						</template>
						<template v-if="item.progress != null" #progress>
							<ProgressBar :progress="item.progress" :max="1" color="blue" full-width />
						</template>
					</Admonition>
				</template>
			</StackedAdmonitions>
		`,
	}),
}

export const KeyboardAndA11y: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition },
		setup() {
			const items = ref<DemoItem[]>([...initialItems])
			function dismiss(id: string) {
				items.value = items.value.filter((i) => i.id !== id)
			}
			return { items, dismiss }
		},
		template: /* html */ `
			<div>
				<p style="color: var(--color-secondary); margin-bottom: 0.75rem; font-size: 0.875rem;">
					Tab to the stack and press <kbd>Enter</kbd> or <kbd>Space</kbd> to expand.
					While collapsed, only the front card's buttons are focusable (inert on back cards).
				</p>
				<StackedAdmonitions :items="items">
					<template #item="{ item }">
						<Admonition
							:type="item.type"
							:header="item.header"
							:body="item.body"
							dismissible
							@dismiss="dismiss(item.id)"
						/>
					</template>
				</StackedAdmonitions>
			</div>
		`,
	}),
}

export const TwoInstances: Story = {
	render: () => ({
		components: { StackedAdmonitions, Admonition },
		setup() {
			const stackA = ref<DemoItem[]>([initialItems[0], initialItems[1]])
			const stackB = ref<DemoItem[]>([initialItems[2]])
			function dismissA(id: string) {
				stackA.value = stackA.value.filter((i) => i.id !== id)
			}
			function dismissB(id: string) {
				stackB.value = stackB.value.filter((i) => i.id !== id)
			}
			return { stackA, stackB, dismissA, dismissB }
		},
		template: /* html */ `
			<div style="display: flex; flex-direction: column; gap: 1.5rem;">
				<StackedAdmonitions :items="stackA">
					<template #item="{ item }">
						<Admonition :type="item.type" :header="item.header" :body="item.body" dismissible @dismiss="dismissA(item.id)" />
					</template>
				</StackedAdmonitions>
				<StackedAdmonitions :items="stackB">
					<template #item="{ item }">
						<Admonition :type="item.type" :header="item.header" :body="item.body" dismissible @dismiss="dismissB(item.id)" />
					</template>
				</StackedAdmonitions>
			</div>
		`,
	}),
}
