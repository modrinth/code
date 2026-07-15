import { AlignLeftIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, group, label, md, stage, toggle } from '../../types/node'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('summary', 'Summary')
		.hint("Is the project's summary sufficient?")
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080bfb5e5c7c6211c693b',
		)
		.icon(AlignLeftIcon)
		.children(
			label(md('checklist/text/summary/summary')),

			group()
				.title('Summary Issues?')
				.children(
					toggle('insufficient', 'Insufficient')
						.enabled((state) => !state.repeat_title)
						.action(action().suggestedStatus('flagged').severity('low').message()),

					toggle('repeat_title', 'Repeat of Title')
						.enabled((state) => !state.insufficient)
						.action(action().suggestedStatus('flagged').severity('low').message()),

					toggle('formatting', 'Formatting').action(
						action().suggestedStatus('flagged').severity('low').message(),
					),

					toggle('non_english', 'Non-english').action(
						action().suggestedStatus('flagged').severity('medium').message(),
					),

					toggle('repeat_ip', 'Repeat of IP')
						.shown(computed(() => !!project.value?.minecraft_server))
						.action(action().suggestedStatus('flagged').severity('medium').message()),
				),
		)
}
