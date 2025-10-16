import { XIcon } from '@modrinth/assets'

import type { Stage } from '../../types/stage'

const undefinedProjectStage: Stage = {
	title: 'This project is undefined!',
	id: 'undefined-project',
	icon: XIcon,
	guidance_url: 'https://modrinth.com/legal/rules#miscellaneous',
	navigate: '/versions',
	shouldShow: (project) => project.versions.length === 0,
	actions: [
		{
			id: 'undefined_no_versions',
			type: 'button',
			label: 'No Versions',
			weight: -100,
			suggestedStatus: 'rejected',
			message: async () =>
				(await import('../messages/undefined-project/no_versions.md?raw')).default,
		},
	],
}

export default undefinedProjectStage
