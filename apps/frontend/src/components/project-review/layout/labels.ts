import type { ProjectReviewSlot } from './types'

export const reviewPanelLabels = {
	project: 'Summary',
	rulings: 'Rulings',
	issues: 'Issues',
	're-review': 'Re-review',
	thread: 'Thread',
	settings: 'Settings',
	description: 'Description',
	gallery: 'Gallery',
	disclosures: 'Disclosures',
	permissions: 'Permissions',
	versions: 'Versions',
	'tech-review': 'Tech Review',
} satisfies Record<ProjectReviewSlot, string>
