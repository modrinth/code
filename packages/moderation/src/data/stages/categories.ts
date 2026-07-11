import { TagsIcon } from '@modrinth/assets'

import { button, group, mdMsg, mdText, prose, stage } from '../../types/node'

export default stage(
	'tags',
	'Tags',
	"Are the project's tags accurate?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0802f96aafc0397a9f6d3',
	{
		icon: TagsIcon,
		navigate: '/settings/tags',
		shown: (_project, projectV3) =>
			(projectV3?.categories.length ?? 0) > 0 ||
			(projectV3?.additional_categories.length ?? 0) > 0,
	},
	[
		prose(mdText('categories')),

		group().children(
			button('inaccurate', 'Inaccurate')
				.weight(700)
				.suggestedStatus('flagged')
				.severity('low')
				.message(mdMsg('categories/inaccurate')),

			button('optimization_misused', 'Optimization')
				.shown(({ project }) =>
					project.categories.includes('optimization') ||
					project.additional_categories.includes('optimization'),
				)
				.weight(701)
				.suggestedStatus('flagged')
				.severity('low')
				.message(async (ctx) => {
					const base = await mdMsg('categories/inaccurate')(ctx)
					const extra = await mdMsg('categories/optimization_misused')(ctx)
					return base + extra
				}),

			button('resolutions_misused', 'Resolutions')
				.shown(({ project }) => project.project_types.includes('resourcepack'))
				.weight(702)
				.suggestedStatus('flagged')
				.severity('low')
				.message(async (ctx) => {
					const base = await mdMsg('categories/inaccurate')(ctx)
					const extra = await mdMsg('categories/resolutions_misused')(ctx)
					return base + extra
				}),
		),
	],
)
