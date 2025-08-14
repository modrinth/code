import { LibraryIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const description: Stage = {
	title: 'Is the description sufficient, accurate, and accessible?',
	id: 'description',
	icon: LibraryIcon,
	guidance_url: 'https://modrinth.com/legal/rules#general-expectations',
	navigate: '/',
	actions: [
		{
			id: 'description_insufficient',
			type: 'button',
			label: 'Insufficient (custom)',
			weight: 400,
			suggestedStatus: 'flagged',
			severity: 'medium',
			message: async () => (await import('../messages/description/insufficient.md?raw')).default,
			relevantExtraInput: [
				{
					label: 'Please elaborate on how the author can improve their description.',
					variable: 'EXPLAINER',
					large: true,
					required: true,
				},
			],
		} as ButtonAction,
		{
			id: 'description_insufficient_packs',
			type: 'button',
			label: 'Insufficient',
			weight: 401,
			suggestedStatus: 'flagged',
			severity: 'medium',
			shouldShow: (project) => project.project_type === 'modpack',
			message: async () =>
				(await import('../messages/description/insufficient-packs.md?raw')).default,
		} as ButtonAction,
		{
			id: 'description_insufficient_projects',
			type: 'button',
			label: 'Insufficient',
			weight: 401,
			suggestedStatus: 'flagged',
			severity: 'medium',
			shouldShow: (project) => project.project_type !== 'modpack',
			message: async () =>
				(await import('../messages/description/insufficient-projects.md?raw')).default,
		} as ButtonAction,
		{
			id: 'description_non_english',
			type: 'button',
			label: 'Non-english',
			weight: 402,
			suggestedStatus: 'flagged',
			severity: 'medium',
			message: async () => (await import('../messages/description/non-english.md?raw')).default,
		} as ButtonAction,
		{
			id: 'description_unfinished',
			type: 'button',
			label: 'Unfinished',
			weight: 403,
			suggestedStatus: 'flagged',
			severity: 'low',
			message: async () => (await import('../messages/description/unfinished.md?raw')).default,
		} as ButtonAction,
		{
			id: 'description_headers_as_body',
			type: 'button',
			label: 'Headers as body text',
			weight: 404,
			suggestedStatus: 'flagged',
			severity: 'low',
			message: async () => (await import('../messages/description/headers-as-body.md?raw')).default,
		} as ButtonAction,
		{
			id: 'description_image_only',
			type: 'button',
			label: 'Image-only',
			weight: 405,
			suggestedStatus: 'flagged',
			severity: 'medium',
			message: async () => (await import('../messages/description/image-only.md?raw')).default,
		} as ButtonAction,
		{
			id: 'description_non_standard_text',
			type: 'button',
			label: 'Non-standard text',
			weight: 406,
			suggestedStatus: 'flagged',
			severity: 'medium',
			message: async () =>
				(await import('../messages/description/non-standard-text.md?raw')).default,
		} as ButtonAction,
		{
			id: 'description_clarity',
			type: 'button',
			label: 'Unclear / Misleading',
			weight: 407,
			suggestedStatus: 'rejected',
			severity: 'high',
			message: async () => (await import('../messages/description/clarity.md?raw')).default,
		} as ButtonAction,
	],
}

export default description
