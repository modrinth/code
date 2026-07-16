import { TagsIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, fix, group, label, md, stage, toggle } from '../../types/node'

const resolutionTags = new Set(['8x-', '16x', '32x', '48x', '64x', '128x', '256x', '512x+'])

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	const inaccurateMsg = md('checklist/messages/tags/inaccurate')
	const optimizationMsg = inaccurateMsg.concat(md('checklist/messages/tags/optimizationMisused'))
	const resolutionsMsg = inaccurateMsg.concat(md('checklist/messages/tags/resolutionsMisused'))

	return stage('tags', 'Tags')
		.hint("Are the project's tags accurate?")
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0802f96aafc0397a9f6d3',
		)
		.icon(TagsIcon)
		.navigate('/settings/tags')
		.shown(
			computed(
				() => project.value.categories.length > 0 || project.value.additional_categories.length > 0,
			),
		)
		.children(
			label(md('checklist/text/categories')),

			group().children(
				toggle('inaccurate', 'Inaccurate').action(
					action().suggestedStatus('flagged').severity('low').message(),
				),

				toggle('optimizationMisused', 'Optimization')
					.shown(
						computed(
							() =>
								project.value.categories.includes('optimization') ||
								project.value.additional_categories.includes('optimization'),
						),
					)
					.action(
						action()
							.suggestedStatus('flagged')
							.severity('low')
							.message(optimizationMsg)
							.fix(
								fix().project((patch) => {
									patch.categories = project.value.categories.filter((c) => c !== 'optimization')
									patch.additional_categories = project.value.additional_categories.filter(
										(c) => c !== 'optimization',
									)
								}),
							),
					),

				toggle('resolutionsMisused', 'Resolutions')
					.shown(computed(() => project.value.project_types.includes('resourcepack')))
					.action(
						action()
							.suggestedStatus('flagged')
							.severity('low')
							.message(resolutionsMsg)
							.fix(
								fix().project((patch) => {
									patch.categories = project.value.categories.filter((c) => !resolutionTags.has(c))
									patch.additional_categories = project.value.additional_categories.filter(
										(c) => !resolutionTags.has(c),
									)
								}),
							),
					),
			),
		)
}
