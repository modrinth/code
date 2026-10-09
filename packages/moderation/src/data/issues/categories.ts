import { TagsIcon } from '@modrinth/assets'

import inaccurateMessage from '../messages/checklist/messages/tags/inaccurate.md'
import optimizationMisusedMessage from '../messages/checklist/messages/tags/optimization-misused.md'
import resolutionsMisusedMessage from '../messages/checklist/messages/tags/resolutions-misused.md'
import { issue, panel, section, select, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'
import type { ReviewContext } from './component-builders/types'

const resolutionTags = new Set(['8x-', '16x', '32x', '48x', '64x', '128x', '256x', '512x+'])

function automaticallyRemovedTags({ projectV3, selected }: ReviewContext) {
	const tags = new Set<string>()
	if (selected.toggleIds.includes('tags-optimization-misused')) tags.add('optimization')
	if (selected.toggleIds.includes('tags-resolutions-misused'))
		for (const tag of [...projectV3.categories, ...projectV3.additional_categories])
			if (resolutionTags.has(tag)) tags.add(tag)
	return [...tags]
}

function removedTags(ctx: ReviewContext) {
	return [...new Set([...ctx.getSelectValues('remove-tags'), ...automaticallyRemovedTags(ctx)])]
}

export const categoriesInaccurateIssue = issue({
	id: 'categories-inaccurate',
	title: 'Inaccurate tags',
	category: 'Tags',
	actions: [issueTargets.removeTags(removedTags)],
	message: (ctx) => {
		const tags = removedTags(ctx)
		return [
			inaccurateMessage,
			ctx.selected.toggleIds.includes('tags-optimization-misused')
				? optimizationMisusedMessage
				: '',
			ctx.selected.toggleIds.includes('tags-resolutions-misused') ? resolutionsMisusedMessage : '',
			tags.length
				? `Please remove the following tags from your project\n\n${tags.map((tag) => `- ${tag}`).join('\n')}`
				: '',
		]
			.filter((message) => message.trim())
			.join('\n\n')
	},
	suggestedStatus: 'flagged',
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
			required: (ctx) => automaticallyRemovedTags(ctx).length === 0,
			options: ({ projectV3 }) =>
				[...new Set([...projectV3.categories, ...projectV3.additional_categories])].map(
					(value) => ({ value, label: value }),
				),
		}),
	),
)
