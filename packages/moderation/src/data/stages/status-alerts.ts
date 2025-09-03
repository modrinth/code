import { TriangleAlertIcon } from '@modrinth/assets'

import type { ButtonAction, DropdownAction, DropdownActionOption } from '../../types/actions'
import type { Stage } from '../../types/stage'

const statusAlerts: Stage = {
	title: `Is anything else affecting this project's status?`,
	id: 'status-alerts',
	icon: TriangleAlertIcon,
	text: async () => (await import('../messages/checklist-text/status-alerts/text.md?raw')).default,
	guidance_url:
		'https://www.notion.so/Project-Modification-Guidelines-22e5ee711bf080628416f0471ba6af02',
	navigate: '/moderation',
	actions: [
		{
			id: 'status_corrections_applied',
			type: 'button',
			label: 'Corrections applied',
			weight: -999999,
			suggestedStatus: 'approved',
			disablesActions: ['status_private_use', 'status_account_issues'],
			shouldShow: (project) => project.status !== 'approved',
			message: async () => (await import('../messages/status-alerts/fixed.md?raw')).default,
		} as ButtonAction,
		{
			id: 'status_corrections_applied-approved',
			type: 'button',
			label: 'Corrections applied',
			weight: -999999,
			suggestedStatus: 'approved',
			disablesActions: ['status_private_use', 'status_account_issues'],
			shouldShow: (project) => project.status === 'approved',
			message: async () =>
				(await import('../messages/status-alerts/fixed-approved.md?raw')).default,
		} as ButtonAction,
		{
			id: 'status_private_use',
			type: 'button',
			label: 'Private use',
			weight: -999999,
			suggestedStatus: 'flagged',
			disablesActions: ['status_corrections_applied', 'status_account_issues'],
			message: async () => (await import('../messages/status-alerts/private.md?raw')).default,
		} as ButtonAction,
		{
			id: 'status_account_issues',
			type: 'button',
			label: 'Account issues',
			weight: -999999,
			suggestedStatus: 'rejected',
			disablesActions: ['status_corrections_applied', 'status_private_use'],
			message: async () =>
				(await import('../messages/status-alerts/account_issues.md?raw')).default,
		} as ButtonAction,
		{
			id: 'status_tec_source_request',
			type: 'button',
			label: `Request Source`,
			suggestedStatus: 'rejected',
			severity: 'critical',
			disablesActions: ['status_corrections_applied', 'status_private_use'],
			shouldShow: (project) =>
				project.project_type === 'mod' ||
				project.project_type === 'shader' ||
				project.project_type.toString() === 'plugin',
			weight: -999999,
			message: async () => '',
			enablesActions: [
				{
					id: 'status_tec_source_request_options',
					type: 'dropdown',
					label: 'Why are you requesting source?',
					options: [
						{
							label: 'Obfuscated',
							weight: 999999,
							message: async () =>
								(await import('../messages/status-alerts/tec/source_request-obfs.md?raw')).default,
						} as DropdownActionOption,
						{
							label: 'Binaries',
							weight: 999000,
							message: async () =>
								(await import('../messages/status-alerts/tec/source_request-bins.md?raw')).default,
						} as DropdownActionOption,
					],
				} as DropdownAction,
			],
		} as ButtonAction,
		{
			id: 'status_automod_confusion',
			type: 'button',
			label: `Automod confusion`,
			weight: -999999,
			message: async () =>
				(await import('../messages/status-alerts/automod_confusion.md?raw')).default,
		} as ButtonAction,
	],
}

export default statusAlerts
