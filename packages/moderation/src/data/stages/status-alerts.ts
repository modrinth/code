import { TriangleAlertIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import type { Ref } from 'vue'
import { computed } from 'vue'

import type { NodeBuilder, NodeState, StageNodeBuilder } from '../../types/node'
import {
	action,
	group,
	isNodeActive,
	label,
	md,
	resolveChildren,
	stage,
	toggle,
	walkNodes,
} from '../../types/node'

export default function (
	mainStages: StageNodeBuilder[],
	globalState: Ref<Record<string, Record<string, NodeState>>>,
) {
	const { projectV3: project } = injectProjectPageContext()

	return stage('status-alerts', 'Status Alerts')
		.hint(`Is anything else affecting this project's status?`)
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080968699c397e470eca6',
		)
		.icon(TriangleAlertIcon)
		.navigate('/moderation')
		.children(
			label(md('checklist/text/status-alerts/text')),

			//TODO: chyz combine these?
			group().children(
				toggle('corrections_applied', 'Corrections applied')
					.shown(computed(() => project.value.status !== 'approved'))
					.action(action().suggestedStatus('approved').message().applyFixes())
					.children((_state) => {
						const fakeChecklist = group().children(...mainStages)
						const result: NodeBuilder[] = []
						walkNodes(
							[fakeChecklist],
							(globalState.value ?? {}) as unknown as Record<string, NodeState>,
							(node, nodeState, localState) => {
								if (!node._action?._fixes?.length || !isNodeActive(node, nodeState)) return
								result.push(...resolveChildren(node, localState))
							},
						)
						return result
					}),

				toggle('corrections_applied_approved', 'Corrections applied')
					.shown(computed(() => project.value.status === 'approved'))
					.action(action().suggestedStatus('approved').message()),

				//TODO: chyz combine these
				toggle('private_use', 'Private use')
					.shown(computed(() => !project.value.minecraft_server))
					.action(action().suggestedStatus('flagged').message()),

				toggle('private_use_server', 'Private community')
					.shown(computed(() => !!project.value.minecraft_server))
					.action(action().suggestedStatus('flagged').message()),

				toggle('server_use', 'Server use')
					.shown(
						computed(
							() =>
								project.value.project_types.includes('modpack') && !project.value.minecraft_server,
						),
					)
					.action(action().message()),

				toggle('account_issues', 'Account issues').action(
					action().suggestedStatus('rejected').message(),
				),

				toggle('automod_confusion', 'Automod confusion')
					.shown(computed(() => !project.value.minecraft_server))
					.action(action().message()),

				toggle('demonetized', 'Demonetized')
					.shown(
						computed(
							() =>
								project.value.monetization_status === 'force-demonetized' &&
								!project.value.project_types.includes('modpack') &&
								!project.value.minecraft_server,
						),
					)
					.action(action().message()),

				toggle('demonetized_modpack', 'Demonetized')
					.shown(
						computed(
							() =>
								project.value.monetization_status === 'force-demonetized' &&
								project.value.project_types.includes('modpack') &&
								!project.value.minecraft_server,
						),
					)
					.action(action().message()),
			),
		)
}
