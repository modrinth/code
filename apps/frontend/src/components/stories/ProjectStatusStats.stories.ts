import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ProjectStatusStats from '../ProjectStatusStats.vue'

const projectStats = [
	{ status: 'tech_review_failed', count: 2 },
	{ status: 'approved', count: 24 },
	{ status: 'unlisted', count: 3 },
	{ status: 'private', count: 4 },
	{ status: 'withheld', count: 2 },
	{ status: 'rejected', count: 5 },
	{ status: 'draft', count: 6 },
	{ status: 'processing', count: 1 },
	{ status: 'archived', count: 7 },
	{ status: 'scheduled', count: 1 },
	{ status: 'unknown', count: 1 },
]
const accountStats = [
	{ status: 'shared_instances_banned', count: 1 },
	{
		status: 'account_restricted',
		count: 1,
		details: ['Create new projects', 'Upload new versions', 'Repeated policy violations'],
	},
	{
		status: 'account_locked',
		count: 1,
		details: 'Account compromised; contact support to restore access.',
	},
]

const meta = {
	title: 'Website/Moderation/ProjectStatusStats',
	component: ProjectStatusStats,
	args: {
		stats: [
			{ status: 'approved', count: 12 },
			{ status: 'rejected', count: 2 },
		],
	},
	parameters: { layout: 'centered' },
} satisfies Meta<typeof ProjectStatusStats>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const AllBadges: StoryObj = {
	render: () => ({
		components: { ProjectStatusStats },
		setup: () => ({ projectStats, accountStats }),
		template: /* html */ `
			<div class="flex max-w-sm flex-col gap-4 p-4">
				<div>
					<p class="m-0 font-semibold text-contrast">Project statuses</p>
					<ProjectStatusStats :stats="projectStats" />
				</div>
				<div>
					<p class="m-0 font-semibold text-contrast">Account restrictions</p>
					<ProjectStatusStats :stats="accountStats" />
				</div>
				<div>
					<p class="m-0 font-semibold text-contrast">Combined</p>
					<ProjectStatusStats :stats="[...projectStats, ...accountStats]" />
				</div>
				<p class="m-0 text-sm text-secondary">Hover or focus a badge to see its label and restriction details.</p>
			</div>
		`,
	}),
}

export const Empty: Story = { args: { stats: [] } }
export const ZeroCounts: Story = {
	args: { stats: [...projectStats, ...accountStats].map((stat) => ({ ...stat, count: 0 })) },
}
