import { SignatureIcon } from '@modrinth/assets'

import { action, toggle, group, md, stage, stageFn } from '../../types/node'

export default stageFn((project) => stage('permissions', 'Modpack Permissions')
	.hint("Does this project's external content have any issues?")
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892')
	.icon(SignatureIcon)
	.navigate('/settings/permissions')
	.shown((project.project_types?.includes('modpack') ?? false) && !project.minecraft_server)
	.children(
		group().children(
			toggle('invalid_permissions', 'Invalid permissions')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/externals-permissions/invalid')),
				),

			toggle('prohibited_external_content', 'Prohibited externals')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/externals-permissions/prohibited')),
				),

			toggle('missing_permissions', 'Missing permissions')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/externals-permissions/missing')),
				),

			toggle('non-commercial-external-content', 'Non-commercial externals')
				.shown(project.monetization_status === 'monetized')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/externals-permissions/non-commercial')),
				),
		),
	),
)
