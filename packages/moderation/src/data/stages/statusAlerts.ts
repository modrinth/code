import { TriangleAlertIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import type { Ref } from 'vue'
import { computed } from 'vue'

import type { NodeBuilder, NodeState, StageNodeBuilder } from '../../types/node'
import {
	action,
	getBooleanChildState,
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

	return stage('statusAlerts', 'Status Alerts')
		.hint(`Is anything else affecting this project's status?`)
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080968699c397e470eca6',
		)
		.icon(TriangleAlertIcon)
		.navigate('/moderation')
		.children(
			label(md('checklist/text/statusAlerts/text')),

			group().children(
				toggle('correctionsApplied', 'Corrections applied')
					.action(
						action()
							.suggestedStatus('approved')
							.message(() => `correctionsApplied${project.value.status === 'approved' ? '-approved' : ''}`)
							.applyFixes(),
					)
					.children(
						computed<NodeBuilder | null>(() => {
							const fixNodes: NodeBuilder[] = []
							walkNodes(
								[group().children(...mainStages)],
								(globalState.value ?? {}) as unknown as Record<string, NodeState>,
								(node, nodeState) => {
									if (!node._action?._fixes?.length) return
									if (!isNodeActive(node, nodeState)) return
									const childState = getBooleanChildState(nodeState)
									fixNodes.push(...resolveChildren(node, childState))
								},
							)
							return fixNodes.length > 0 ? group().children(...fixNodes) : null
						}),
					),

				//TODO: chyz combine these
				toggle('privateUse', 'Private use')
					.shown(computed(() => !project.value.minecraft_server))
					.action(action().suggestedStatus('flagged').message()),

				toggle('privateUse-server', 'Private community')
					.shown(computed(() => !!project.value.minecraft_server))
					.action(action().suggestedStatus('flagged').message()),

				toggle('serverUse', 'Server use')
					.shown(
						computed(
							() =>
								project.value.project_types.includes('modpack') && !project.value.minecraft_server,
						),
					)
					.action(action().message()),

				toggle('accountIssues', 'Account issues').action(
					action().suggestedStatus('rejected').message(),
				),

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

				toggle('demonetized-modpack', 'Demonetized')
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
