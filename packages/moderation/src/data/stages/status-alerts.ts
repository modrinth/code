import { TriangleAlertIcon } from '@modrinth/assets'

import type { NodeBuilder, NodeState } from '../../types/node'
import { action, toggle, group, isNodeActive, label, md, stage, walkNodes } from '../../types/node'
import checklist from '../checklist'

export default stage('status-alerts', 'Status Alerts')
	.hint(`Is anything else affecting this project's status?`)
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080968699c397e470eca6')
	.icon(TriangleAlertIcon)
	.navigate('/moderation')
	.children(
		label(md('checklist/text/status-alerts/text')),

		group().children(
			toggle('corrections_applied', 'Corrections applied')
				.shown(({ project }) => project.status !== 'approved')
				.action(
					action()

						.suggestedStatus('approved')
						.message(md('checklist/messages/status-alerts/fixed'))
						.applyFixes(),
				)
				.children(ctx => {
					const result: NodeBuilder[] = []
					walkNodes([checklist], ctx.globalState as unknown as Record<string, NodeState>, ctx, (node, state) => {
						if (!node._action?._fixes?.length || !isNodeActive(node, state)) return
						result.push(...node._children)
					})
					return result
				}),

			toggle('corrections_applied_approved', 'Corrections applied')
				.shown(({ project }) => project.status === 'approved')
				.action(
					action()

						.suggestedStatus('approved')
						.message(md('checklist/messages/status-alerts/fixed-approved')),
				),

			toggle('private_use', 'Private use')
				.shown(({ project }) => !project.minecraft_server)
				.action(
					action()

						.suggestedStatus('flagged')
						.message(md('checklist/messages/status-alerts/private/private')),
				),

			toggle('private_use_server', 'Private community')
				.shown(({ project }) => !!project.minecraft_server)
				.action(
					action()

						.suggestedStatus('flagged')
						.message(md('checklist/messages/status-alerts/private/private-server')),
				),

			toggle('server_use', 'Server use')
				.shown(
					({ project }) => project.project_types.includes('modpack') && !project.minecraft_server,
				)
				.action(
					action()

						.message(md('checklist/messages/status-alerts/serverpack')),
				),

			toggle('account_issues', 'Account issues')
				.action(
					action()

						.suggestedStatus('rejected')
						.message(md('checklist/messages/status-alerts/account_issues')),
				),

			toggle('automod_confusion', 'Automod confusion')
				.shown(({ project }) => !project.minecraft_server)
				.action(
					action()

						.message(md('checklist/messages/status-alerts/automod_confusion')),
				),

			toggle('demonetized', 'Demonetized')
				.shown(
					({ project }) =>
						project.monetization_status === 'force-demonetized' &&
						!project.project_types.includes('modpack') &&
						!project.minecraft_server,
				)
				.action(
					action()

						.message(md('checklist/messages/status-alerts/demonetized/demonetized')),
				),

			toggle('demonetized_modpack', 'Demonetized')
				.shown(
					({ project }) =>
						project.monetization_status === 'force-demonetized' &&
						project.project_types.includes('modpack') &&
						!project.minecraft_server,
				)
				.action(
					action()

						.message(md('checklist/messages/status-alerts/demonetized/demonetized-modpack')),
				),
		),
	)
