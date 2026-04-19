import { ImageIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const gallery: Stage = {
	title: "Are this project's gallery images sufficient?",
	id: 'gallery',
	icon: ImageIcon,
	guidance_url:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08096828bd1c3f24d8b8e',
	navigate: '/gallery',
	actions: [
		{
			id: 'gallery_insufficient',
			type: 'button',
			label: 'Insufficient',
			weight: 900,
			suggestedStatus: 'flagged',
			severity: 'low',
			shouldShow: (project, projectV3) => !projectV3?.minecraft_server,
			message: async () => (await import('../messages/gallery/insufficient.md?raw')).default,
		} as ButtonAction,
		{
			id: 'gallery_not_relevant',
			type: 'button',
			label: 'Not relevant',
			weight: 901,
			suggestedStatus: 'flagged',
			severity: 'low',
			shouldShow: (project) => project.gallery && project.gallery.length > 0,
			message: async () => (await import('../messages/gallery/not-relevant.md?raw')).default,
		} as ButtonAction,
	],
}

export default gallery
