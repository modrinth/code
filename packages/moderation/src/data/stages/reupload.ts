import { CopyrightIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const reupload: Stage = {
	title: 'Does the author have proper permissions to post this project?',
	id: 'reupload',
	icon: CopyrightIcon,
	guidance_url: 'https://modrinth.com/legal/rules',
	actions: [
		{
			id: 'reupload_reupload',
			type: 'button',
			label: 'Re-upload',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			message: async () => (await import('../messages/reupload/reupload.md?raw')).default,
			disablesActions: [
				'reupload_unclear_fork',
				'reupload_insufficient_fork',
				'reupload_request_proof',
				'reupload_identity_verification',
			],
			relevantExtraInput: [
				{
					label: 'What is the title of the original project?',
					variable: 'ORIGINAL_PROJECT',
					required: true,
					suggestions: ['Vanilla Tweaks'],
				},
				{
					label: 'What is the author of the original project?',
					variable: 'ORIGINAL_AUTHOR',
					required: true,
					suggestions: ['Vanilla Tweaks Team'],
				},
			],
		} as ButtonAction,
		{
			id: 'reupload_unclear_fork',
			type: 'button',
			label: 'Unclear Fork',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			message: async () => (await import('../messages/reupload/fork.md?raw')).default,
			disablesActions: [
				'reupload_reupload',
				'reupload_insufficient_fork',
				'reupload_request_proof',
				'reupload_identity_verification',
			],
		} as ButtonAction,
		{
			id: 'reupload_insufficient_fork',
			type: 'button',
			label: 'Insufficient Fork',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			message: async () => (await import('../messages/reupload/insufficient_fork.md?raw')).default,
			disablesActions: [
				'reupload_unclear_fork',
				'reupload_reupload',
				'reupload_request_proof',
				'reupload_identity_verification',
			],
		} as ButtonAction,
		{
			id: 'reupload_request_proof',
			type: 'button',
			label: 'Proof of permissions',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			message: async () =>
				(await import('../messages/reupload/proof_of_permissions.md?raw')).default,
			disablesActions: [
				'reupload_reupload',
				'reupload_unclear_fork',
				'reupload_insufficient_fork',
				'reupload_identity_verification',
			],
		},
		{
			id: 'reupload_identity_verification',
			type: 'button',
			label: 'Verify Identity',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			message: async () =>
				(await import('../messages/reupload/identity_verification.md?raw')).default,
			relevantExtraInput: [
				{
					label: 'Where else can the project be found?',
					variable: 'PLATFORM',
					required: true,
				},
			],
			disablesActions: [
				'reupload_reupload',
				'reupload_insufficient_fork',
				'reupload_request_proof',
			],
		},
	],
}

export default reupload
