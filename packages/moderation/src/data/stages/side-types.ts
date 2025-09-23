import { GlobeIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const sideTypes: Stage = {
	title: "Is the project's environment information accurate?",
	id: 'environment',
	icon: GlobeIcon,
	guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
	navigate: '/settings/environment',
	text: async () => (await import('../messages/checklist-text/side_types.md?raw')).default,
	actions: [
		{
			id: 'side_types_inaccurate_modpack',
			type: 'button',
			label: 'Inaccurate',
			weight: 800,
			suggestedStatus: 'flagged',
			severity: 'low',
			shouldShow: (project) => project.project_type === 'modpack',
			message: async () =>
				(await import('../messages/side-types/inaccurate-modpack.md?raw')).default,
		} as ButtonAction,
		{
			id: 'side_types_inaccurate_mod',
			type: 'button',
			label: 'Inaccurate',
			weight: 800,
			suggestedStatus: 'flagged',
			severity: 'low',
			shouldShow: (project) => project.project_type === 'mod',
			message: async () => (await import('../messages/side-types/inaccurate-mod.md?raw')).default,
		} as ButtonAction,
	],
}

export default sideTypes
