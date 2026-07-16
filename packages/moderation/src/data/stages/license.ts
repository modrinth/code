import { BookTextIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, check, group, label, md, type MessageFn, stage, toggle } from '../../types/node'

const licensesNotRequiringSource: string[] = [
	'LicenseRef-All-Rights-Reserved',
	'Apache-2.0',
	'BSD-2-Clause',
	'BSD-3-Clause',
	'CC0-1.0',
	'CC-BY-4.0',
	'CC-BY-SA-4.0',
	'CC-BY-NC-4.0',
	'CC-BY-NC-SA-4.0',
	'CC-BY-ND-4.0',
	'CC-BY-NC-ND-4.0',
	'ISC',
	'MIT',
	'Zlib',
]

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	const noSourceForkMsg: MessageFn = md('checklist/messages/license/noSource-fork')
	const noSourceMsg: MessageFn = md('checklist/messages/license/noSource')

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
				toggle('invalidLink', 'Invalid Link')
					.shown(computed(() => !!project.value.license?.url))
					.action(action().suggestedStatus('flagged').severity('medium').message())
					.children(
						check('customLicense', 'Invalid Link: Custom License').action(action().message()),
					),

				toggle('noSource', 'No Source')
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
