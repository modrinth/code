import { AlignLeftIcon } from '@modrinth/assets'

import { action, button, group, label, md, stage } from '../../types/node'

export default stage(
	'summary',
	'Summary',
	"Is the project's summary sufficient?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080bfb5e5c7c6211c693b',
)
	.icon(AlignLeftIcon)
	.children(
		label(md('checklist/text/summary/summary')),

		group().children(
			button('insufficient', 'Insufficient')
				.enabled(({ state }) => !state['repeat_title'])
				.action(
					action()
						.weight(300)
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/summary/insufficient')),
				),

			button('repeat_title', 'Repeat of Title')
				.enabled(({ state }) => !state['insufficient'])
				.action(
					action()
						.weight(300)
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/summary/repeat-title')),
				),

			button('formatting', 'Formatting')
				.action(
					action()
						.weight(301)
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/summary/formatting')),
				),

			button('non_english', 'Non-english')
				.action(
					action()
						.weight(302)
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/summary/non-english')),
				),

			button('repeat_ip', 'Repeat of IP')
				.shown(({ project }) => !!project?.minecraft_server)
				.action(
					action()
						.weight(303)
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/summary/repeat-ip')),
				),
		),
	)
