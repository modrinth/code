import { ImageIcon } from '@modrinth/assets'

import { button, group, mdMsg, stage } from '../../types/node'

export default stage(
	'gallery',
	'Gallery',
	"Are this project's gallery images sufficient?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08096828bd1c3f24d8b8e',
	[
		group().children(
			button('insufficient', 'Insufficient')
				.shown(({ project }) => !project.minecraft_server)
				.weight(900)
				.suggestedStatus('flagged')
				.severity('low')
				.message(mdMsg('gallery/insufficient')),

			button('not_relevant', 'Not relevant')
				.shown(({ project }) => project.gallery.length > 0)
				.weight(901)
				.suggestedStatus('flagged')
				.severity('low')
				.message(mdMsg('gallery/not-relevant')),
		),
	],
	{
		icon: ImageIcon,
		navigate: '/gallery',
	},
)
