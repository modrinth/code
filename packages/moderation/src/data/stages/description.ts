import { LibraryIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'

import { action, group, markdown, md, stage, toggle } from '../../types/node'

const SHOW_SPOILER_ADVICE = ['horror']

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('description', 'Description')
		.hint('Is the description sufficient, accurate, and accessible?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080508042e70089dd787e',
		)
		.icon(LibraryIcon)
		.navigate('/')
		.children(
			group()
				.title('Description Issues?')
				.children(
					toggle('insufficient', 'Insufficient')
						.action(
							action()
								.suggestedStatus('flagged')
								.severity('medium')
								.message(async (state) => {
									const reasons = state?.reason instanceof Set ? state.reason : new Set<string>()

									let message = await md(
										'checklist/messages/description/insufficient/header',
										(s) => ({ CUSTOM_ADVICE: s.custom?.explainer }),
									)(state)

									if (reasons.size === 0)
										message += await md(
											`checklist/messages/description/insufficient/default/${project.value?.minecraft_java_server ? 'servers' : project.value?.project_types?.includes('modpack') ? 'packs' : 'projects'}`,
										)(state)

									if (reasons.has('fork'))
										message += await md('checklist/messages/description/insufficient/piece/fork')(
											state,
										)

									if (reasons.has('unfinished'))
										message += await md(
											'checklist/messages/description/insufficient/piece/unfinished',
										)(state)

									if (reasons.has('horror'))
										message += await md('checklist/messages/description/insufficient/piece/horror')(
											state,
										)

									// Always put this at bottom
									if (SHOW_SPOILER_ADVICE.some((reason) => reasons.has(reason)))
										message += await md(
											'checklist/messages/description/insufficient/piece/spoiler-guide',
										)(state)

									return message
								}),
						)
						.children(
							group()
								.title('Why is this Description Insufficient?')
								.multiSelect('reason')
								.children(
									toggle('fork', 'Fork'),
									toggle('unfinished', 'Unfinished'),
									toggle('horror', 'Horror'),
									toggle('custom', 'Custom').children(
										markdown('explainer')
											.title('How can the author improve their description?')
											.required(),
									),
								),
						),

					toggle('non_english', 'Non-english')
						.action(
							action()
								.suggestedStatus('flagged')
								.severity('medium')
								.message(
									md(
										() =>
											`checklist/messages/description/non_english${project.value.minecraft_java_server ? '_server' : ''}`,
									),
								),
						)
						.shown(
							computed(() => {
								if (
									!!project.value?.minecraft_java_server &&
									!project.value.minecraft_server?.languages?.includes('en')
								) {
									return false
								} else {
									return true
								}
							}),
						),

					toggle('headers_as_body', 'Headers as body text').action(
						action().suggestedStatus('flagged').severity('low').message(),
					),

					toggle('image_only', 'Image-only').action(
						action().suggestedStatus('flagged').severity('medium').message(),
					),

					toggle('non_standard_text', 'Non-standard text').action(
						action().suggestedStatus('flagged').severity('medium').message(),
					),

					toggle('clarity', 'Unclear / Misleading').action(
						action().suggestedStatus('rejected').severity('high').message(),
					),
				),
		)
}
