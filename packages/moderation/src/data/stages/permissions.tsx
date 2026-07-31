import { SignatureIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { group, stage, toggle } from '../../types/node'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('permissions', 'Modpack Permissions')
		.hint("Does this project's external content have any issues?")
		.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892')
		.icon(SignatureIcon)
		.navigate('/settings/permissions')
		.shown(
			computed(
				() =>
					(project.value.project_types?.includes('modpack') ?? false) &&
					!project.value.minecraft_server,
			),
		)
		.children(
			group().children(
				toggle('invalid-permissions', 'Invalid permissions')
					.suggestedStatus('rejected')
					.severity('high')
					.message(),

				toggle('prohibited-external-content', 'Prohibited externals')
					.suggestedStatus('rejected')
					.severity('high')
					.message(),

				toggle('missing-permissions', 'Missing permissions')
					.suggestedStatus('rejected')
					.severity('high')
					.message(),

				toggle('non-commercial-external-content', 'Non-commercial externals')
					.shown(computed(() => project.value.monetization_status === 'monetized'))
					.suggestedStatus('rejected')
					.severity('high')
					.message(),
			),
		)
}
