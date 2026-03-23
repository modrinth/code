import { ListBulletedIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const ruleFollowing: Stage = {
	title: 'Does this project violate the rules?',
	id: 'rule-following',
	icon: ListBulletedIcon,
	guidance_url:
		'https://www.notion.so/Creator-Communication-Guide-1b65ee711bf080ec9337e3ccdded146c',
	navigate: '/moderation',
	actions: [
		{
			id: 'rule_breaking_yes',
			type: 'button',
			label: 'Yes',
			weight: 0,
			suggestedStatus: 'rejected',
			severity: 'critical',
			message: async () => (await import('../messages/rule-breaking.md?raw')).default,
			relevantExtraInput: [
				{
					label: 'Please explain to the user how it infringes on our content rules.',
					variable: 'MESSAGE',
					required: true,
					large: true,
				},
			],
		} as ButtonAction,
		{
			id: 'paid_access_server',
			type: 'button',
			label: 'Paid access server',
			weight: 0,
			suggestedStatus: 'rejected',
			severity: 'critical',
			shouldShow(project, projectV3) {
				return !!projectV3?.minecraft_server
			},
			message: async () => (await import('../messages/paid-access-server.md?raw')).default,
		},
		{
			id: 'excessive_languages',
			type: 'button',
			label: 'Excessive languages',
			weight: 0,
			suggestedStatus: 'flagged',
			severity: 'low',
			shouldShow(project, projectV3) {
				return (
					!!projectV3?.minecraft_server &&
					!!projectV3?.minecraft_server?.languages?.length &&
					projectV3?.minecraft_server?.languages?.length > 4
				)
			},
			message: async () =>
				(await import('../messages/misc-metadata/excessive_languages-server.md?raw')).default,
		},
	],
}

export default ruleFollowing
