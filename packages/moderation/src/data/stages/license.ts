import { BookTextIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, check, group, label, md, type MessageFn, stage, toggle } from '../../types/node'
import { licensesNotRequiringSource } from '../../utils'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	const noSourceForkMsg: MessageFn = md('checklist/messages/license/no-source-fork')
	const noSourceMsg: MessageFn = md('checklist/messages/license/no-source')

	return stage('license', 'License')
		.hint('Is this license and link valid?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080f8805df7d012a8f770',
		)
		.icon(BookTextIcon)
		.navigate('/settings/license')
		.shown(computed(() => !project.value?.minecraft_server))
		.children(
			label(md('checklist/text/licensing')),

			group().children(
				toggle('invalid-link', 'Invalid Link')
					.shown(computed(() => !!project.value.license?.url))
					.action(action().suggestedStatus('flagged').severity('medium').message())
					.children(
						check('custom-license', 'Invalid Link: Custom License').action(action().message()),
					),

				toggle('no-source', 'No Source')
					.shown(
						computed(() => !licensesNotRequiringSource.includes(project.value.license?.id ?? '')),
					)
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('medium')
							.message(async (state) => {
								if (state.fork) return noSourceForkMsg(state)
								return noSourceMsg(state)
							}),
					)
					.children(check('fork', 'No Source: Fork').action(action().severity('high'))),
			),
		)
}
