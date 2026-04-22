import { GlobeIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../../types/actions'
import type { Stage } from '../../../types/stage'

const environmentMultiple: Stage = {
	title: "Is the project's environment information accurate?",
	id: 'environment',
	navigate: '/settings/versions',
	icon: GlobeIcon,
	guidance_url:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e25ee711bf0802d9a9bdb82dce040eb',
	text: async () =>
		(await import('../../messages/checklist-text/environment/environment-multiple.md?raw')).default,
	shouldShow: (project, projectV3) =>
		(projectV3?.environment?.length ?? 0) !== 1 && !projectV3?.minecraft_server,
	actions: [
		{
			id: 'side_types_inaccurate',
			type: 'button',
			label: 'Inaccurate',
			weight: 800,
			suggestedStatus: 'flagged',
			severity: 'low',
			shouldShow: (project) => project.project_type === 'mod' || project.project_type === 'modpack',
			message: async () => (await import('../../messages/environment/inaccurate.md?raw')).default,
		} as ButtonAction,
	],
}

export default environmentMultiple
