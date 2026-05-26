import { SignatureIcon } from '@modrinth/assets'

import type { Stage } from '../../types/stage'

const permissions: Stage = {
	title: 'Does this projects external content have any issues?',
	id: 'permissions',
	icon: SignatureIcon,
	guidance_url: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	navigate: '/settings/permissions',
	actions: [
		{
			id: 'invalid-permissions',
			type: 'button',
			label: 'Invalid permissions',
			weight: 2000,
			suggestedStatus: 'rejected',
			severity: 'high',
			shouldShow: (project, projectV3) =>
				projectV3.project_types?.includes('modpack') && !projectV3?.minecraft_server,
			message: async () =>
				(await import('../messages/externals-permissions/invalid.md?raw')).default,
		},
		{
			id: 'prohibited-extrernal-content',
			type: 'button',
			label: 'Prohibited externals',
			weight: 2001,
			suggestedStatus: 'rejected',
			severity: 'high',
			shouldShow: (project, projectV3) =>
				projectV3.project_types?.includes('modpack') && !projectV3?.minecraft_server,
			message: async () =>
				(await import('../messages/externals-permissions/prohibited.md?raw')).default,
		},
		{
			id: 'missing-permissions',
			type: 'button',
			label: 'Missing permissions',
			weight: 2002,
			suggestedStatus: 'rejected',
			severity: 'high',
			shouldShow: (project, projectV3) =>
				projectV3.project_types?.includes('modpack') && !projectV3?.minecraft_server,
			message: async () =>
				(await import('../messages/externals-permissions/missing.md?raw')).default,
		},
	],
}

export default permissions
