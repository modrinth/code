import { XIcon } from '@modrinth/assets'

import type { Stage } from '../../types/stage'

const undefinedProjectStage: Stage = {
	title: 'This project is undefined!',
	id: 'undefined-project',
	icon: XIcon,
	guidance_url:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080018bf3d822a2f51a35',
	navigate: '/versions',
	shouldShow: (project, projectV3) => project.versions.length === 0 && !projectV3?.minecraft_server,
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
