import { TriangleAlertIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import type { Ref } from 'vue'
import { computed } from 'vue'

import type { NodeBuilder, NodeState, StageNodeBuilder } from '../../types/node'
import {
	getBooleanChildState,
	group,
	isNodeActive,
	label,
	resolveChildren,
	stage,
	toggle,
	walkNodes,
} from '../../types/node'
import { Priorities } from '../priorities.ts'

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
			label('text'),

			group().children(
				toggle('corrections-applied', 'Corrections applied')
					.suggestedStatus('approved')
					.message(
						() => `corrections-applied${project.value.status === 'approved' ? '-approved' : ''}`,
					)
					//TODO this is temporary
					.priority(Priorities.alerts)
					.applyFixes()
					.children(
						computed<NodeBuilder | null>(() => {
							const fixNodes: NodeBuilder[] = []
							walkNodes(
								[group().children(...mainStages)],
								(globalState.value ?? {}) as unknown as Record<string, NodeState>,
								(node, nodeState) => {
									if (!node._fixes.length) return
									if (!isNodeActive(node, nodeState)) return
									const childState = getBooleanChildState(nodeState)
									fixNodes.push(...resolveChildren(node, childState))
								},
							)
							return fixNodes.length > 0 ? group().children(...fixNodes) : null
						}),
					),

				//TODO: chyz combine these
				toggle('private-use', 'Private use')
					.shown(computed(() => !project.value.minecraft_server))
					.suggestedStatus('flagged')
					.message()
					.priority(Priorities.alerts),

				toggle('private-use-server', 'Private community')
					.shown(computed(() => !!project.value.minecraft_server))
					.suggestedStatus('flagged')
					.message()
					.priority(Priorities.alerts),

				toggle('server-use', 'Server use')
					.shown(
						computed(
							() =>
								project.value.project_types.includes('modpack') && !project.value.minecraft_server,
						),
					)
					.message(),

				toggle('account-issues', 'Account issues').suggestedStatus('rejected').message(),

				toggle('demonetized', 'Demonetized')
					.shown(
						computed(
							() =>
								project.value.monetization_status === 'force-demonetized' &&
								!project.value.project_types.includes('modpack') &&
								!project.value.minecraft_server,
						),
					)
					.message()
					.priority(Priorities.alerts),

				toggle('demonetized-modpack', 'Demonetized')
					.shown(
						computed(
							() =>
								project.value.monetization_status === 'force-demonetized' &&
								project.value.project_types.includes('modpack') &&
								!project.value.minecraft_server,
						),
					)
					.message()
					.priority(Priorities.alerts),
			),
		)
}
