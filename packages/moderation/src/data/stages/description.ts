import { LibraryIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, group, markdown, md, stage, toggle } from '../../types/node'

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
									const reasons =
										state?.reason instanceof Set ? state.reason : new Set<string>()

									let message = await md('checklist/messages/description/insufficient/header')(state)

									if (reasons.size === 0) {
										const typePath = project.value?.minecraft_java_server
											? 'servers'
											: project.value?.project_types?.includes('modpack')
												? 'packs'
												: 'projects'
										message += await md(
											`checklist/messages/description/insufficient/${typePath}`,
										)(state)
									}

									if (reasons.has('fork'))
										message += await md('checklist/messages/description/insufficient/fork')(state)

									if (reasons.has('custom'))
										message += await md(
											'checklist/messages/description/insufficient/custom',
											(s) => ({ EXPLAINER: s.explainer }),
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
									toggle('custom', 'Custom').children(
										markdown('explainer')
											.title('How can the author improve their description?')
											.required(),
									),
								),
						),

					//TODO: combine these 2 non-englishes
					toggle('non_english', 'Non-english')
						.shown(computed(() => !project.value.minecraft_java_server))
						.action(action().suggestedStatus('flagged').severity('medium').message()),

					toggle('non_english_server', 'Non-english')
						.shown(computed(() => !!project.value.minecraft_java_server))
						.action(action().suggestedStatus('flagged').severity('medium').message()),

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
