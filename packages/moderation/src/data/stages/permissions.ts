import { SignatureIcon } from '@modrinth/assets'

import { button, group, mdMsg, stage } from '../../types/node'

const isModpack = ({ project }: { project: { project_types: string[]; minecraft_server?: unknown } }) =>
	project.project_types.includes('modpack') && !project.minecraft_server

export default stage(
	'permissions',
	'Modpack Permissions',
	'Does this project\'s external content have any issues?',
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	{ icon: SignatureIcon, navigate: '/settings/permissions' },
	[
		group().children(
			button('invalid_permissions', 'Invalid permissions')
				.shown(isModpack)
				.weight(2000)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('externals-permissions/invalid')),

			button('prohibited_external_content', 'Prohibited externals')
				.shown(isModpack)
				.weight(2001)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('externals-permissions/prohibited')),

			button('missing_permissions', 'Missing permissions')
				.shown(isModpack)
				.weight(2002)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('externals-permissions/missing')),
		),
	],
)
