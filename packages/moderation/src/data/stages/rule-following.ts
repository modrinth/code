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
	],
}

export default ruleFollowing
