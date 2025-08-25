import { ImageIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const gallery: Stage = {
	title: "Are this project's gallery images sufficient?",
	id: 'gallery',
	icon: ImageIcon,
	guidance_url: 'https://modrinth.com/legal/rules#general-expectations',
	navigate: '/gallery',
	actions: [
		{
			id: 'gallery_insufficient',
			type: 'button',
			label: 'Insufficient',
			weight: 900,
			suggestedStatus: 'flagged',
			severity: 'low',
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
