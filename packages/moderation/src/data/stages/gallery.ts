import { ImageIcon } from '@modrinth/assets'

import { action, toggle, group, md, stage } from '../../types/node'

export default stage('gallery', 'Gallery')
	.hint("Are this project's gallery images sufficient?")
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08096828bd1c3f24d8b8e')
	.icon(ImageIcon)
	.navigate('/gallery')
	.children(
		group().children(
			toggle('insufficient', 'Insufficient')
				.shown(({ project }) => !project.minecraft_server)
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/gallery/insufficient')),
				),

			toggle('not_relevant', 'Not relevant')
				.shown(({ project }) => project.gallery.length > 0)
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/gallery/not-relevant')),
				),
		),
	)
