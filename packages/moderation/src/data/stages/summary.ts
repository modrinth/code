import { AlignLeftIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const summary: Stage = {
	title: "Is the project's summary sufficient?",
	text: async () => (await import('../messages/checklist-text/summary/summary.md?raw')).default,
	id: 'summary',
	icon: AlignLeftIcon,
	guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
	actions: [
		{
			id: 'summary_insufficient',
			type: 'button',
			label: 'Insufficient',
			weight: 300,
			suggestedStatus: 'flagged',
			severity: 'low',
			disablesActions: ['summary_repeat_title'],
			message: async () => (await import('../messages/summary/insufficient.md?raw')).default,
		} as ButtonAction,
		{
			id: 'summary_repeat_title',
			type: 'button',
			label: 'Repeat of title',
			weight: 300,
			suggestedStatus: 'flagged',
			severity: 'low',
			disablesActions: ['summary_insufficient'],
			message: async () => (await import('../messages/summary/repeat-title.md?raw')).default,
		} as ButtonAction,
		{
			id: 'summary_formatting',
			type: 'button',
			label: 'Formatting',
			weight: 301,
			suggestedStatus: 'flagged',
			severity: 'low',
			message: async () => (await import('../messages/summary/formatting.md?raw')).default,
		} as ButtonAction,
		{
			id: 'summary_non_english',
			type: 'button',
			label: 'Non-english',
			weight: 302,
			suggestedStatus: 'flagged',
			severity: 'medium',
			message: async () => (await import('../messages/summary/non-english.md?raw')).default,
		} as ButtonAction,
	],
}

export default summary
