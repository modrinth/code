import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'

import ProjectPublishingBanner from '../../components/project/ProjectPublishingBanner.vue'

const itemsRemaining = { state: 'todo', remaining: 3, completed: 6 } as const
const oneItemRemaining = { state: 'todo', remaining: 1, completed: 8 } as const

const meta = {
	title: 'Project/Publishing Banner',
	component: ProjectPublishingBanner,
	args: itemsRemaining,
} satisfies Meta<typeof ProjectPublishingBanner>
export default meta
type Story = StoryObj<typeof meta>

const examples = [
	{ state: 'checking' },
	{ state: 'unavailable' },
	itemsRemaining,
	oneItemRemaining,
	{ state: 'ready' },
	{ state: 'review' },
	{ state: 'approved' },
	{ state: 'unlisted' },
	{ state: 'private' },
	{ state: 'changes', remaining: 2, completed: 4 },
	{ state: 'ready-to-resubmit' },
	{ state: 'withheld' },
	{ state: 'default' },
] satisfies Pick<
	InstanceType<typeof ProjectPublishingBanner>['$props'],
	'state' | 'remaining' | 'completed'
>[]

export const AllStates: Story = {
	render: () => ({
		components: { ProjectPublishingBanner },
		setup: () => ({ examples }),
		template: `
			<div class="flex flex-col gap-4">
				<ProjectPublishingBanner
					v-for="example in examples"
					:key="example.state + ':' + example.remaining"
					v-bind="example"
				/>
			</div>
		`,
	}),
}

export const ItemsRemaining: Story = { args: itemsRemaining }
export const OneItemRemaining: Story = { args: oneItemRemaining }
export const Checking: Story = { args: { state: 'checking' } }
export const ChecksUnavailable: Story = { args: { state: 'unavailable' } }
export const ReadyForReview: Story = { args: { state: 'ready', remaining: 0, onSubmit: fn() } }
export const UnderReview: Story = { args: { state: 'review' } }
export const Approved: Story = { args: { state: 'approved' } }
export const ApprovedUnlisted: Story = { args: { state: 'unlisted' } }
export const ApprovedPrivate: Story = { args: { state: 'private' } }
export const ChangesRequested: Story = { args: { state: 'changes' } }
export const ReadyToResubmit: Story = {
	args: { state: 'ready-to-resubmit', remaining: 0, onSubmit: fn() },
}
export const UnlistedByStaff: Story = { args: { state: 'withheld' } }
export const Default: Story = { args: { state: 'default' } }
export const DrawerSummary: Story = { args: { state: 'review', compact: true } }
export const AllDrawerSummaries: Story = {
	render: () => ({
		components: { ProjectPublishingBanner },
		setup: () => ({ examples }),
		template: `
			<div class="flex max-w-[400px] flex-col gap-6">
				<ProjectPublishingBanner
					v-for="example in examples"
					:key="example.state + ':' + example.remaining"
					v-bind="example"
					compact
				/>
			</div>
		`,
	}),
}
