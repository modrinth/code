import { TagsIcon } from '@modrinth/assets'

import inaccurateMessage from '../messages/checklist/messages/tags/inaccurate.md'
import optimizationMisusedMessage from '../messages/checklist/messages/tags/optimization-misused.md'
import resolutionsMisusedMessage from '../messages/checklist/messages/tags/resolutions-misused.md'
import { issue, panel, section, select, toggle } from './component-builders/builders'

const resolutionTags = new Set(['8x-', '16x', '32x', '48x', '64x', '128x', '256x', '512x+'])

export const categoriesInaccurateIssue = issue({
	id: 'categories-inaccurate',
	title: 'Inaccurate tags',
	category: 'Tags',
	message: ({ selected }) =>
		[
			inaccurateMessage,
			selected.toggleIds.includes('tags-optimization-misused') ? optimizationMisusedMessage : '',
			selected.toggleIds.includes('tags-resolutions-misused') ? resolutionsMisusedMessage : '',
		].join('\n'),
	suggestedStatus: 'flagged',
	corrections: ({ projectV3, selected, getSelectValues }) => {
		const remove = new Set(getSelectValues('remove-tags'))
		if (selected.toggleIds.includes('tags-optimization-misused')) remove.add('optimization')
		if (selected.toggleIds.includes('tags-resolutions-misused'))
			for (const tag of resolutionTags) remove.add(tag)
		if (!remove.size) return {}
		return {
			project: {
				categories: projectV3.categories.filter((tag) => !remove.has(tag)),
				additional_categories: projectV3.additional_categories.filter((tag) => !remove.has(tag)),
			},
		}
	},
})

export const categoriesReviewPanel = panel({
	title: 'Tags',
	hint: "Are the project's tags accurate?",
	icon: TagsIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0802f96aafc0397a9f6d3',
	shown: ({ projectV3 }) =>
		projectV3.categories.length > 0 || projectV3.additional_categories.length > 0,
}).content(
	toggle({ issue: categoriesInaccurateIssue, label: 'Inaccurate' }),
	toggle({
		issue: categoriesInaccurateIssue,
		label: 'Optimization',
		id: 'tags-optimization-misused',
		shown: ({ projectV3 }) =>
			[...projectV3.categories, ...projectV3.additional_categories].includes('optimization'),
	}),
	toggle({
		issue: categoriesInaccurateIssue,
		label: 'Resolutions',
		id: 'tags-resolutions-misused',
		shown: ({ projectV3 }) => projectV3.project_types.includes('resourcepack'),
	}),
	section({
		shown: ({ selected }) =>
			selected.issueIds.includes(categoriesInaccurateIssue.id) ||
			selected.toggleIds.includes('tags-optimization-misused') ||
			selected.toggleIds.includes('tags-resolutions-misused'),
	}).content(
		select({
			issue: categoriesInaccurateIssue,
			id: 'remove-tags',
			label: 'Remove inaccurate tags',
			multiple: true,
			options: ({ projectV3 }) =>
				[...new Set([...projectV3.categories, ...projectV3.additional_categories])].map(
					(value) => ({ value, label: value }),
				),
		}),
	),
)
