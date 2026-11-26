import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { fn } from 'storybook/test'
import { ref } from 'vue'

import { isProjectChecklistVisible } from '~/providers/project-publishing'

import OpenDrawerButton from './open-drawer-button.vue'
import ProjectPublishingBanner from './project-publishing-banner.vue'

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
	{ state: 'ready' },
	{ state: 'review' },
	{ state: 'changes', remaining: 2, completed: 4 },
	{ state: 'ready-to-resubmit' },
	{ state: 'withheld' },
	{ state: 'approved' },
] satisfies Pick<
	InstanceType<typeof ProjectPublishingBanner>['$props'],
	'state' | 'remaining' | 'completed'
>[]

export const AllStates: Story = {
	name: 'All states',
	parameters: { layout: 'fullscreen' },
	render: () => ({
		components: { OpenDrawerButton, ProjectPublishingBanner },
		setup: () => ({
			examples,
			dismissed: ref<Record<string, boolean>>({}),
			isProjectChecklistVisible,
		}),
		template: `
			<div class="flex flex-col gap-4 py-4 pl-4">
				<div class="sticky top-0 z-10 grid grid-cols-[minmax(0,2fr)_minmax(0,1fr)_7rem] gap-4 bg-surface-1 py-2">
					<div>
						<h2 class="m-0 text-base font-semibold text-contrast">Publishing banner</h2>
						<p class="mb-0 mt-1 text-sm text-secondary">
							Shown at top of project settings
						</p>
					</div>
					<div>
						<h2 class="m-0 text-base font-semibold text-contrast">Drawer summary</h2>
						<p class="mb-0 mt-1 text-sm text-secondary">
							Shown at top of moderation drawer in checklist tab
						</p>
					</div>
				<h2 class="m-0 text-right text-base font-semibold text-contrast">Drawer launcher</h2>
				</div>
				<div
					v-for="example in examples"
					:key="example.state + ':' + example.remaining"
					class="grid grid-cols-[minmax(0,2fr)_minmax(0,1fr)_7rem] items-start gap-4"
				>
					<div>
						<ProjectPublishingBanner
							v-if="!dismissed[example.state]"
							v-bind="example"
							@dismiss="dismissed[example.state] = true"
						/>
					</div>
					<div>
						<ProjectPublishingBanner
							v-if="isProjectChecklistVisible(example.state, example.remaining ?? 0)"
							v-bind="example"
							compact
						/>
					</div>
					<OpenDrawerButton
						class="justify-self-end"
						:state="example.state"
						:remaining="example.remaining ?? 0"
					/>
				</div>
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
export const DrawerSummary: Story = { args: { state: 'review', compact: true } }
