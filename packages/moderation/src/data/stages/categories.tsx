import { TagsIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { fix, group, md, stage, toggle } from '../../types/node'
import { arrayOrNone } from '../../utils'

const resolutionTags = new Set(['8x-', '16x', '32x', '48x', '64x', '128x', '256x', '512x+'])

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	const inaccurateMsg = md('checklist/messages/tags/inaccurate')
	const optimizationMsg = inaccurateMsg.concat(md('checklist/messages/tags/optimization-misused'))
	const resolutionsMsg = inaccurateMsg.concat(md('checklist/messages/tags/resolutions-misused'))

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
			() => (
				<div class="markdown-body w-full">
					<strong>Featured Tags:</strong> {arrayOrNone(project.value.categories)}
					<br />
					<strong>Additional Tags:</strong> {arrayOrNone(project.value.additional_categories)}
				</div>
			),

			group().children(
				toggle('inaccurate', 'Inaccurate').suggestedStatus('flagged').severity('low').message(),

				toggle('optimization-misused', 'Optimization')
					.shown(
						computed(
							() =>
								project.value.categories.includes('optimization') ||
								project.value.additional_categories.includes('optimization'),
						),
					)
					.suggestedStatus('flagged')
					.severity('low')
					.rawMessage(optimizationMsg)
					.fix(
						fix().project((patch) => {
							patch.categories = project.value.categories.filter((c) => c !== 'optimization')
							patch.additional_categories = project.value.additional_categories.filter(
								(c) => c !== 'optimization',
							)
						}),
					),

				toggle('resolutions-misused', 'Resolutions')
					.shown(computed(() => project.value.project_types.includes('resourcepack')))
					.suggestedStatus('flagged')
					.severity('low')
					.rawMessage(resolutionsMsg)
					.fix(
						fix().project((patch) => {
							patch.categories = project.value.categories.filter((c) => !resolutionTags.has(c))
							patch.additional_categories = project.value.additional_categories.filter(
								(c) => !resolutionTags.has(c),
							)
						}),
					),
			),
		)
}
