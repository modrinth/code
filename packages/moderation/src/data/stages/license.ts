import { BookTextIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { check, group, label, md, type MessageFn, stage, toggle } from '../../types/node'
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
			label('licensing'),

			group().children(
				toggle('invalid-link', 'Invalid Link')
					.shown(computed(() => !!project.value.license?.url))
					.suggestedStatus('flagged')
					.severity('medium')
					.message()
					.children(check('custom-license', 'Invalid Link: Custom License').message())
					.collect(),

				toggle('no-source', 'No Source')
					.shown(
						computed(() => !licensesNotRequiringSource.includes(project.value.license?.id ?? '')),
					)
					.suggestedStatus('rejected')
					.severity('medium')
					.rawMessage(async (state) => {
						if (state.fork) return noSourceForkMsg(state)
						return noSourceMsg(state)
					})
					.children(check('fork', 'No Source: Fork').severity('high')),
			),
		)
}
