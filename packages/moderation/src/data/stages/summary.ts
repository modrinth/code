import { AlignLeftIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { group, label, stage, toggle } from '../../types/node'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('summary', 'Summary')
		.hint("Is the project's summary sufficient?")
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080bfb5e5c7c6211c693b',
		)
		.icon(AlignLeftIcon)
		.children(
			label('summary'),

			group()
				.title('Summary Issues?')
				.children(
					toggle('insufficient', 'Insufficient')
						.enabled((state) => !state['repeat-title'])
						.suggestedStatus('flagged')
						.severity('low')
						.message(),

					toggle('repeat-title', 'Repeat of Title')
						.enabled((state) => !state.insufficient)
						.suggestedStatus('flagged')
						.severity('low')
						.message(),

					toggle('formatting', 'Formatting').suggestedStatus('flagged').severity('low').message(),

					toggle('non-english', 'Non-english')
						.suggestedStatus('flagged')
						.severity('medium')
						.message()
						.shown(
							computed(() => {
								if (
									!!project.value?.minecraft_java_server &&
									!project.value.minecraft_server?.languages?.includes('en')
								) {
									return false
								} else {
									return true
								}
							}),
						),

					toggle('repeat-ip', 'Repeat of IP')
						.shown(computed(() => !!project.value?.minecraft_server))
						.suggestedStatus('flagged')
						.severity('medium')
						.message(),
				),
		)
}
