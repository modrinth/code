import { SignatureIcon } from '@modrinth/assets'

import { action, button, group, md, stage } from '../../types/node'

export default stage(
	'permissions',
	'Modpack Permissions',
	"Does this project's external content have any issues?",
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
)
	.icon(SignatureIcon)
	.navigate('/settings/permissions')
	.shown(
		({ project }) =>
			(project.project_types?.includes('modpack') ?? false) && !project.minecraft_server,
	)
	.children(
		group().children(
			button('invalid_permissions', 'Invalid permissions')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/externals-permissions/invalid')),
				),

			button('prohibited_external_content', 'Prohibited externals')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/externals-permissions/prohibited')),
				),

			button('missing_permissions', 'Missing permissions')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/externals-permissions/missing')),
				),

			button('non-commercial-external-content', 'Non-commercial externals')
				.shown(({ project }) => project.monetization_status === 'monetized')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/externals-permissions/non-commercial')),
				),
		),
	)
