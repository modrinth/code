import { SignatureIcon } from '@modrinth/assets'

import { button, group, mdMsg, stage } from '../../types/node'

export default stage(
	'permissions',
	'Modpack Permissions',
	"Does this project's external content have any issues?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	{
		icon: SignatureIcon,
		navigate: '/settings/permissions',
		shown: (_project, projectV3) =>
			(projectV3?.project_types?.includes('modpack') ?? false) && !projectV3?.minecraft_server,
	},
	[
		group().children(
			button('invalid_permissions', 'Invalid permissions')
				.weight(2000)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('externals-permissions/invalid')),

			button('prohibited_external_content', 'Prohibited externals')
				.weight(2001)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('externals-permissions/prohibited')),

			button('missing_permissions', 'Missing permissions')
				.weight(2002)
				.suggestedStatus('rejected')
				.severity('high')
				.message(mdMsg('externals-permissions/missing')),
		),
	],
)
