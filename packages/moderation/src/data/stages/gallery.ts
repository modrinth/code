import { ImageIcon } from '@modrinth/assets'

import { action, button, group, md, stage } from '../../types/node'

export default stage(
	'gallery',
	'Gallery',
	"Are this project's gallery images sufficient?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08096828bd1c3f24d8b8e',
)
	.icon(ImageIcon)
	.navigate('/gallery')
	.children(
		group().children(
			button('insufficient', 'Insufficient')
				.shown(({ project }) => !project.minecraft_server)
				.action(
					action()
						.weight(900)
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/gallery/insufficient')),
				),

			button('not_relevant', 'Not relevant')
				.shown(({ project }) => project.gallery.length > 0)
				.action(
					action()
						.weight(901)
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/gallery/not-relevant')),
				),
		),
	)
