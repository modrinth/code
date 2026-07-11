import { AlignLeftIcon } from '@modrinth/assets'

import { button, group, mdMsg, mdText, prose, stage } from '../../types/node'

export default stage(
	'summary',
	'Summary',
	"Is the project's summary sufficient?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080bfb5e5c7c6211c693b',
	[
		prose(mdText('summary/summary')),

		group().children(
			button('insufficient', 'Insufficient')
				.enabled(({ state }) => !state['repeat_title'])
				.weight(300)
				.suggestedStatus('flagged')
				.severity('low')
				.message(mdMsg('summary/insufficient')),

			button('repeat_title', 'Repeat of Title')
				.enabled(({ state }) => !state['insufficient'])
				.weight(300)
				.suggestedStatus('flagged')
				.severity('low')
				.message(mdMsg('summary/repeat-title')),

			button('formatting', 'Formatting')
				.weight(301)
				.suggestedStatus('flagged')
				.severity('low')
				.message(mdMsg('summary/formatting')),

			button('non_english', 'Non-english')
				.weight(302)
				.suggestedStatus('flagged')
				.severity('medium')
				.message(mdMsg('summary/non-english')),

			button('repeat_ip', 'Repeat of IP')
				.weight(303)
				.suggestedStatus('flagged')
				.severity('medium')
				.shown(({ project }) => !!project?.minecraft_server)
				.message(mdMsg('summary/repeat-ip')),
		),
	],
	{ icon: AlignLeftIcon },
)
