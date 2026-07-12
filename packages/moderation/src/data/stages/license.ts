import { BookTextIcon } from '@modrinth/assets'

import { action, button, group, label, md, stage, toggle } from '../../types/node'

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

export default stage(
	'license',
	'License',
	'Is this license and link valid?',
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080f8805df7d012a8f770',
)
	.icon(BookTextIcon)
	.navigate('/settings/license')
	.shown(({ project }) => !project?.minecraft_server)
	.children(
		label(md('checklist/text/licensing')),

		group().children(
			button('invalid_link', 'Invalid Link')
				.shown(({ project }) => !!project.license?.url)
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/license/invalid_link')),
				)
				.children(
					toggle('custom_license', 'Invalid Link: Custom License')
						.action(
							action()
								.message(md('checklist/messages/license/invalid_link-custom_license')),
						),
				),

			button('no_source', 'No Source')
				.shown(({ project }) => !licensesNotRequiringSource.includes(project.license?.id ?? ''))
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('medium')
						.message(async (ctx) => {
							if (ctx.state.fork) return md('checklist/messages/license/no_source-fork')(ctx)
							return md('checklist/messages/license/no_source')(ctx)
						}),
				)
				.children(
					toggle('fork', 'No Source: Fork')
						.action(action().severity('high')),
				),
		),
	)
