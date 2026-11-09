import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectPublishingBanner from '../../components/project/ProjectPublishingBanner.vue'

const meta = {
	title: 'Project/Publishing Banner',
	component: ProjectPublishingBanner,
	args: { state: 'todo', remaining: 6 },
} satisfies Meta<typeof ProjectPublishingBanner>
export default meta
type Story = StoryObj<typeof meta>

export const ItemsRemaining: Story = { args: { state: 'todo', remaining: 6 } }
export const OneItemRemaining: Story = { args: { state: 'todo', remaining: 1 } }
export const Checking: Story = { args: { state: 'checking' } }
export const ChecksUnavailable: Story = { args: { state: 'unavailable' } }
export const ReadyForReview: Story = { args: { state: 'ready', remaining: 0 } }
export const UnderReview: Story = { args: { state: 'review' } }
export const Approved: Story = { args: { state: 'approved' } }
export const ApprovedUnlisted: Story = { args: { state: 'unlisted' } }
export const ApprovedPrivate: Story = { args: { state: 'private' } }
export const ChangesRequested: Story = { args: { state: 'changes' } }
export const UnlistedByStaff: Story = { args: { state: 'withheld' } }
export const Default: Story = { args: { state: 'default' } }
export const AllStates: Story = {
	render: () => ({
		components: { ProjectPublishingBanner },
		setup: () => ({
			states: [
				'checking',
				'unavailable',
				'todo',
				'ready',
				'review',
				'approved',
				'unlisted',
				'private',
				'changes',
				'withheld',
				'default',
			],
		}),
		template: `<div class="flex flex-col gap-4"><ProjectPublishingBanner v-for="state in states" :key="state" :state="state" :remaining="6" /></div>`,
	}),
}

export const DrawerSummary: Story = { args: { state: 'review', compact: true } }
export const AllDrawerSummaries: Story = {
	render: () => ({
		components: { ProjectPublishingBanner },
		setup: () => ({
			states: [
				'checking',
				'unavailable',
				'todo',
				'ready',
				'review',
				'approved',
				'unlisted',
				'private',
				'changes',
				'withheld',
				'default',
			],
		}),
		template: `<div class="flex max-w-[400px] flex-col gap-6"><ProjectPublishingBanner v-for="state in states" :key="state" :state="state" :remaining="6" compact /></div>`,
	}),
}
