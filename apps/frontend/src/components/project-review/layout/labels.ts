import type { ProjectReviewSlot } from './types'

export const reviewPanelLabels = {
	left: 'Project details',
	right: 'Right panel',
	description: 'Description',
	gallery: 'Gallery',
	disclosures: 'Disclosures',
	permissions: 'Permissions',
	versions: 'Versions',
	'tech-review': 'Tech Review',
	slicer: 'Slicer',
} satisfies Record<ProjectReviewSlot, string>
