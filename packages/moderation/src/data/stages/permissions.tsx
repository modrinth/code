import type { Labrinth } from '@modrinth/api-client'
import { SignatureIcon } from '@modrinth/assets'
import { injectModrinthClient, injectProjectPageContext } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { group, stage, toggle } from '../../types/node'

function isResolved(attributionGroup: Labrinth.Attribution.Internal.AttributionGroup): boolean {
	const attribution = attributionGroup.attribution
	if (!attribution) return false
	if (attribution.kind === 'globally_allowed') return true
	return attribution.moderation_status?.kind === 'approved'
}

export default function () {
	const { projectV3: project } = injectProjectPageContext()
	const { labrinth } = injectModrinthClient()

	const { data: attributionData } = useQuery({
		queryKey: ['project-attribution', project.value.id],
		queryFn: () => labrinth.attribution_internal.listProjectAttribution(project.value.id),
	})

	const unresolvedCount = computed(
		() => (attributionData.value ?? []).filter((g) => !isResolved(g)).length,
	)

	return stage('permissions', 'Modpack Permissions')
		.hint("Does this project's external content have any issues?")
		.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892')
		.icon(SignatureIcon)
		.navigate('/settings/permissions')
		.shown(
			computed(
				() =>
					(project.value.project_types?.includes('modpack') ?? false) &&
					!project.value.minecraft_server &&
					unresolvedCount.value > 0,
			),
		)
		.sticky()
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
