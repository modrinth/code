import { LibraryIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, group, markdown, md, option, stage, toggle } from '../../types/node'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	const insufficientCustomMsg = md(
		'checklist/messages/description/insufficient/custom',
		(state) => ({
			EXPLAINER: state.explainer,
		}),
	)
	const insufficientHeaderMsg = md('checklist/messages/description/insufficient/header')
	const insufficientForkMsg = md('checklist/messages/description/insufficient/fork')
	const insufficientServersMsg = md('checklist/messages/description/insufficient/servers')
	const insufficientPacksMsg = md('checklist/messages/description/insufficient/packs')
	const insufficientProjectsMsg = md('checklist/messages/description/insufficient/projects')

	return stage('description', 'Description')
		.hint('Is the description sufficient, accurate, and accessible?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080508042e70089dd787e',
		)
		.icon(LibraryIcon)
		.navigate('/')
		.children(
			//TODO: coolbot which parts of this go in insufficient vs top level
			group()
				.title('Description Issues?')
				.children(
					toggle('insufficient', 'Insufficient')
						.action(
							action()
								.suggestedStatus('flagged')
								.severity('medium')
								.message(async (state) => {
									const reason = state?.reason as string | undefined
									if (reason === 'custom') {
										return insufficientCustomMsg(state)
									}
									if (reason === 'fork') {
										const header = await insufficientHeaderMsg(state)
										const detail = await insufficientForkMsg(state)
										return `${header}\n\n${detail}`
									}
									if (project.value?.minecraft_java_server) return insufficientServersMsg(state)
									if (project.value?.project_types?.includes('modpack'))
										return insufficientPacksMsg(state)
									return insufficientProjectsMsg(state)
								}),
						)
						.children(
							group()
								.title('Why is this Description Insufficient?')
								//TODO: coolbot should multiple be allowed here an we just make 1 message with all issues?
								.singleSelect('reason')
								.children(
									option('fork', 'Fork'),
									option('custom', 'Custom').children(
										markdown('explainer')
											.title('How can the author improve their description?')
											.required(),
									),
								),
						),

					//TODO chyz combine these 2 non-englishes
					toggle('non_english', 'Non-english')
						.shown(computed(() => !project.value.minecraft_java_server))
						.action(action().suggestedStatus('flagged').severity('medium').message()),

					toggle('non_english_server', 'Non-english')
						.shown(computed(() => !!project.value.minecraft_java_server))
						.action(action().suggestedStatus('flagged').severity('medium').message()),

					toggle('unfinished', 'Unfinished').action(
						action().suggestedStatus('flagged').severity('low').message(),
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
