import { TagsIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const categories: Stage = {
	title: "Are the project's tags accurate?",
	id: 'tags',
	icon: TagsIcon,
	guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
	navigate: '/settings/tags',
	shouldShow: (project) =>
		project.categories.length > 0 || project.additional_categories.length > 0,
	text: async () => {
		return (await import('../messages/checklist-text/categories.md?raw')).default
	},
	actions: [
		{
			id: 'categories_inaccurate',
			type: 'button',
			label: 'Inaccurate',
			weight: 700,
			suggestedStatus: 'flagged',
			severity: 'low',
			message: async () => (await import('../messages/categories/inaccurate.md?raw')).default,
			disablesActions: ['categories_optimization_misused', 'categories_resolutions_misused'],
		} as ButtonAction,
		{
			id: 'categories_optimization_misused',
			type: 'button',
			label: 'Optimization',
			weight: 701,
			suggestedStatus: 'flagged',
			severity: 'low',
			shouldShow: (project) =>
				project.categories.includes('optimization') ||
				project.additional_categories.includes('optimization'),
			message: async () =>
				(await import('../messages/categories/inaccurate.md?raw')).default +
				(await import('../messages/categories/optimization_misused.md?raw')).default,
			disablesActions: ['categories_inaccurate', 'categories_resolutions_misused'],
		} as ButtonAction,
		{
			id: 'categories_resolutions_misused',
			type: 'button',
			label: 'Resolutions',
			weight: 702,
			suggestedStatus: 'flagged',
			severity: 'low',
			shouldShow: (project) => project.project_type === 'resourcepack',
			message: async () =>
				(await import('../messages/categories/inaccurate.md?raw')).default +
				(await import('../messages/categories/resolutions_misused.md?raw')).default,
			disablesActions: ['categories_inaccurate', 'categories_optimization_misused'],
		},
	],
}

export default categories
