import { ImageIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, group, stage, toggle } from '../../types/node'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('gallery', 'Gallery')
		.hint("Are this project's gallery images sufficient?")
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08096828bd1c3f24d8b8e',
		)
		.icon(ImageIcon)
		.navigate('/gallery')
		.children(
			group().children(
				toggle('insufficient', 'Insufficient').action(
					action().suggestedStatus('flagged').severity('low').message(),
				),

				toggle('not_relevant', 'Not relevant')
					.shown(computed(() => project.value.gallery.length > 0))
					.action(action().suggestedStatus('flagged').severity('low').message()),

				toggle('showcase_clarity', 'Showcase Clarity').action(
					action().suggestedStatus('rejected').severity('high').message(),
				),
			),
		)
}
