import { ScaleIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const postApproval: Stage = {
	title: 'Issue warnings, notices, or takedowns?',
	id: 'post-approval',
	icon: ScaleIcon,
	guidance_url: 'https://modrinth.com/legal/rules',
	shouldShow: (project) => project.status === 'approved',
	actions: [
		{
			id: 'issue_warning',
			type: 'button',
			label: 'Issue warning',
			weight: 3000,
			suggestedStatus: 'approved',
			severity: 'low',
			message: async () => (await import('../messages/post-approval/issue-warning.md?raw')).default,
		},
		{
			id: 'missed_deadline',
			type: 'button',
			label: 'Missed due date',
			weight: -999,
			suggestedStatus: 'flagged',
			severity: 'high',
			message: async () =>
				(await import('../messages/post-approval/missed-deadline.md?raw')).default,
			disablesActions: ['issue_warning', 'metadata_issue'],
			relevantExtraInput: [
				{
					label: 'What status is the project being set to?',
					variable: 'STATUS',
					required: true,
				},
			],
		},
		{
			id: 'metadata_issue',
			type: 'button',
			label: 'Incorrect metadata',
			weight: 0,
			suggestedStatus: 'approved',
			severity: 'low',
			message: async () =>
				(await import('../messages/post-approval/metadata-issue.md?raw')).default,
			enablesActions: [
				{
					id: 'dependencies',
					type: 'button',
					label: 'Missing Dependencies',
					weight: 1,
					severity: 'low',
					message: async () =>
						(await import('../messages/misc-metadata/dependencies.md?raw')).default,
					relevantExtraInput: [
						{
							label: 'Dependency Name',
							variable: 'DEPENDENCY_NAME',
							required: true,
						},
						{
							label: 'Dependency Link',
							variable: 'DEPENDENCY_LINK',
							required: true,
						},
					],
				},
				{
					id: 'mc_versions',
					type: 'button',
					label: 'Game versions',
					weight: 2,
					severity: 'low',
					message: async () =>
						(await import('../messages/misc-metadata/mc-versions.md?raw')).default,
					relevantExtraInput: [
						{
							label: 'Provide more details about game versions issue?',
							variable: 'SPECIFICS',
							required: false,
							large: true,
						},
					],
				},
				{
					id: 'loaders',
					type: 'button',
					label: 'Loaders',
					weight: 3,
					severity: 'low',
					message: async () => (await import('../messages/misc-metadata/loaders.md?raw')).default,
					relevantExtraInput: [
						{
							label: 'Provide more details about loaders issue?',
							variable: 'SPECIFICS',
							required: false,
							large: true,
						},
					],
				},
				{
					id: 'license',
					type: 'button',
					label: 'Inconsistent Licensing',
					weight: 4,
					severity: 'low',
					message: async () =>
						(await import('../messages/misc-metadata/inconsistent-license.md?raw')).default,
				},
			],
		} as ButtonAction,
	],
}

export default postApproval
