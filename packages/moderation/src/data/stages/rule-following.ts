import { ListBulletedIcon } from '@modrinth/assets'

import type { ButtonAction, MultiSelectChipsAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const ruleFollowing: Stage = {
	title: 'Does this project violate the rules?',
	id: 'rule-following',
	icon: ListBulletedIcon,
	guidance_url:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080709084f6269835607f',
	navigate: '/moderation',
	actions: [
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
			message: async () =>
				(await import('../messages/checklist-messages/paid-access-server.md?raw')).default,
		},
		{
			id: 'prohibited_content',
			type: 'button',
			label: 'Prohibited Content',
			weight: 100,
			suggestedStatus: 'rejected',
			severity: 'critical',
			message: async () =>
				(
					await import('../messages/checklist-messages/rule-breaking/prohibited-content-header.md?raw')
				).default.trimEnd(),
			enablesActions: [
				{
					id: 'prohibited_content_options',
					type: 'multi-select-chips',
					label: 'Which Prohibited Content rules does this project violate?',
					joinWith: '\n',
					options: [
						{
							label: 'Objectionable',
							weight: 101,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/objectionable.md?raw')
								).default,
						},
						{
							label: 'Discriminatory or Explicit',
							weight: 102,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/discriminatory.md?raw')
								).default,
						},
						{
							label: 'IP Infringement',
							weight: 103,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/ip-infringement.md?raw')
								).default,
						},
						{
							label: 'Rights Violation',
							weight: 104,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/legal-rights.md?raw')
								).default,
						},
						{
							label: 'Illegal Activity',
							weight: 105,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/illegal-activity.md?raw')
								).default,
						},
						{
							label: 'Harmful or Deceptive',
							weight: 106,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/harmful.md?raw')
								).default,
						},
						{
							label: 'Misleading claims',
							weight: 107,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/misleading.md?raw')
								).default,
						},
						{
							label: 'Impersonation',
							weight: 108,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/impersonation.md?raw')
								).default,
						},
						{
							label: 'False Endorsement',
							weight: 109,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/false-endorsement.md?raw')
								).default,
						},
						{
							label: 'Profanity',
							weight: 110,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/profanity.md?raw')
								).default,
						},
						{
							label: 'Undisclosed Data Upload',
							weight: 111,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/undisclosed-upload.md?raw')
								).default,
						},
						{
							label: 'Mojang Bypass',
							weight: 112,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/prohibited-content/mojang-bypass.md?raw')
								).default,
						},
					],
				} as MultiSelectChipsAction,
			],
		} as ButtonAction,
		{
			id: 'cheat_or_hack_advertising',
			type: 'button',
			label: 'Hacks',
			weight: 150,
			suggestedStatus: 'rejected',
			severity: 'critical',
			message: async () =>
				(
					await import('../messages/checklist-messages/rule-breaking/cheat-or-hack-advertising.md?raw')
				).default,
		} as ButtonAction,
		{
			id: 'server_side_opt_out',
			type: 'button',
			label: 'Opt-out',
			weight: 200,
			suggestedStatus: 'flagged',
			severity: 'high',
			message: async () =>
				(await import('../messages/checklist-messages/rule-breaking/server-side-opt-out.md?raw'))
					.default,
		} as ButtonAction,
		{
			id: 'server_side_opt_in',
			type: 'button',
			label: 'Opt-in',
			weight: 300,
			suggestedStatus: 'flagged',
			severity: 'high',
			message: async () =>
				(
					await import('../messages/checklist-messages/rule-breaking/server-side-opt-in-header.md?raw')
				).default.trimEnd(),
			enablesActions: [
				{
					id: 'server_side_opt_in_options',
					type: 'multi-select-chips',
					label: 'Which features of this project require a Server-side Opt-in?',
					joinWith: '\n',
					options: [
						{
							label: 'X-ray',
							weight: 301,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/server-side-opt-in/x-ray.md?raw')
								).default,
						},
						{
							label: 'Aim Assist',
							weight: 302,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/server-side-opt-in/aim-bot.md?raw')
								).default,
						},
						{
							label: 'Movement',
							weight: 303,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/server-side-opt-in/movement.md?raw')
								).default,
						},
						{
							label: 'PvP',
							weight: 304,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/server-side-opt-in/pvp.md?raw')
								).default,
						},
						{
							label: 'Anti 3.x',
							weight: 305,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/server-side-opt-in/hiding-mods.md?raw')
								).default,
						},
						{
							label: 'Dupe',
							weight: 306,
							message: async () =>
								(
									await import('../messages/checklist-messages/rule-breaking/server-side-opt-in/item-duplication.md?raw')
								).default,
						},
					],
				} as MultiSelectChipsAction,
			],
		} as ButtonAction,
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
				(
					await import('../messages/checklist-messages/misc-metadata/excessive_languages-server.md?raw')
				).default,
		},
		{
			id: 'rule_breaking_other',
			type: 'button',
			label: 'Other',
			weight: 0,
			suggestedStatus: 'rejected',
			severity: 'critical',
			message: async () =>
				(await import('../messages/checklist-messages/rule-breaking.md?raw')).default,
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
