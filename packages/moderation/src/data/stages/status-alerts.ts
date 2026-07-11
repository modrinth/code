import { TriangleAlertIcon } from '@modrinth/assets'

import { button, group, mdMsg, mdText, prose, stage } from '../../types/node'

export default stage(
	'status-alerts',
	'Status Alerts',
	`Is anything else affecting this project's status?`,
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080968699c397e470eca6',
	{ icon: TriangleAlertIcon, navigate: '/moderation' },
	[
		prose(mdText('status-alerts/text')),

		group().children(
			button('corrections_applied', 'Corrections applied')
				.shown(({ projectV2 }) => projectV2.status !== 'approved')
				.weight(-999999)
				.suggestedStatus('approved')
				.message(mdMsg('status-alerts/fixed')),

			button('corrections_applied_approved', 'Corrections applied')
				.shown(({ projectV2 }) => projectV2.status === 'approved')
				.weight(-999999)
				.suggestedStatus('approved')
				.message(mdMsg('status-alerts/fixed-approved')),

			button('private_use', 'Private use')
				.shown(({ project }) => !project.minecraft_server)
				.weight(-999999)
				.suggestedStatus('flagged')
				.message(mdMsg('status-alerts/private/private')),

			button('private_use_server', 'Private community')
				.shown(({ project }) => !!project.minecraft_server)
				.weight(-999999)
				.suggestedStatus('flagged')
				.message(mdMsg('status-alerts/private/private-server')),

			button('server_use', 'Server use')
				.shown(
					({ project }) => project.project_types.includes('modpack') && !project.minecraft_server,
				)
				.weight(-999999)
				.message(mdMsg('status-alerts/serverpack')),

			button('account_issues', 'Account issues')
				.weight(-999999)
				.suggestedStatus('rejected')
				.message(mdMsg('status-alerts/account_issues')),

			button('automod_confusion', 'Automod confusion')
				.shown(({ project }) => !project.minecraft_server)
				.weight(-999999)
				.message(mdMsg('status-alerts/automod_confusion')),

			button('status_demonetized', 'Demonetized')
				.shown(
					({ project }) =>
						project.monetization_status === 'force-demonetized' &&
						!project.project_types?.includes('modpack'),
				)
				.weight(-999999)
				.message(mdMsg('status-alerts/demonetized')),

			button('status_demonetized-modpack', 'Demonetized')
				.shown(
					({ project }) =>
						project.monetization_status === 'force-demonetized' &&
						project.project_types?.includes('modpack'),
				)
				.weight(-999999)
				.message(mdMsg('status-alerts/demonetized-modpack')),
		),
	],
)
