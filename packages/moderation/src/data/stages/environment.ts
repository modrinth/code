import { GlobeIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const environment: Stage = {
	title: "Is the project's environment information accurate?",
	id: 'environment',
	navigate: '/settings/environment',
	icon: GlobeIcon,
	guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
	text: async (project, projectV3Partial) => {
		if (projectV3Partial?.environment.length === 1) {
			return (await import('../messages/checklist-text/environment/environment.md?raw')).default
		} else {
			return (await import('../messages/checklist-text/environment/environment-multiple.md?raw'))
				.default
		}
	},
	shouldShow: () => true,
	actions: [
		{
			id: 'side_types_inaccurate',
			type: 'button',
			label: 'Inaccurate',
			weight: 800,
			suggestedStatus: 'flagged',
			severity: 'low',
			shouldShow: (project) => project.project_type === 'mod' || project.project_type === 'modpack',
			message: async () => (await import('../messages/environment/inaccurate.md?raw')).default,
		} as ButtonAction,
	],
}

export default environment
