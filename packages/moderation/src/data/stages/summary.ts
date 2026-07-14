import { AlignLeftIcon } from '@modrinth/assets'

import { action, toggle, group, label, md, stage, stageFn } from '../../types/node'

export default stageFn((project) => stage('summary', 'Summary')
	.hint("Is the project's summary sufficient?")
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080bfb5e5c7c6211c693b')
	.icon(AlignLeftIcon)
	.children(
		label(md('checklist/text/summary/summary')),

		group().children(
			toggle('insufficient', 'Insufficient')
				.enabled(({ state }) => !state['repeat_title'])
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/summary/insufficient')),
				),

			toggle('repeat_title', 'Repeat of Title')
				.enabled(({ state }) => !state['insufficient'])
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/summary/repeat-title')),
				),

			toggle('formatting', 'Formatting')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/summary/formatting')),
				),

			toggle('non_english', 'Non-english')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/summary/non-english')),
				),

			toggle('repeat_ip', 'Repeat of IP')
				.shown(!!project?.minecraft_server)
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/summary/repeat-ip')),
				),
		),
	),
)
