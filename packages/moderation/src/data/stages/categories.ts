import { TagsIcon } from '@modrinth/assets'

import { action, toggle, group, label, md, stage, stageFn } from '../../types/node'

export default stageFn((project) => stage('tags', 'Tags')
	.hint("Are the project's tags accurate?")
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0802f96aafc0397a9f6d3')
	.icon(TagsIcon)
	.navigate('/settings/tags')
	.shown(project.categories.length > 0 || project.additional_categories.length > 0)
	.children(
		label(md('checklist/text/categories')),

		group().children(
			toggle('inaccurate', 'Inaccurate')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/categories/inaccurate')),
				),

			toggle('optimization_misused', 'Optimization')
				.shown(
					project.categories.includes('optimization') ||
					project.additional_categories.includes('optimization'),
				)
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(async (ctx) => {
							const base = await md('checklist/messages/categories/inaccurate')(ctx)
							const extra = await md('checklist/messages/categories/optimization_misused')(ctx)
							return base + extra
						}),
				),

			toggle('resolutions_misused', 'Resolutions')
				.shown(project.project_types.includes('resourcepack'))
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(async (ctx) => {
							const base = await md('checklist/messages/categories/inaccurate')(ctx)
							const extra = await md('checklist/messages/categories/resolutions_misused')(ctx)
							return base + extra
						}),
				),
		),
	),
)
