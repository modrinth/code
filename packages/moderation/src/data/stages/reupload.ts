import { CopyrightIcon } from '@modrinth/assets'

import type { ButtonAction, ToggleAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const reupload: Stage = {
	title: 'Does the author have proper permissions to post this project?',
	id: 'reupload',
	icon: CopyrightIcon,
	guidance_url:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080d1a0a2cda3ff2ce997',
	actions: [
		{
			id: 'reupload_reupload',
			type: 'button',
			label: 'Re-upload',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			shouldShow: (project, projectV3) => !projectV3?.minecraft_server,
			message: async () => (await import('../messages/reupload/reupload.md?raw')).default,
			disablesActions: [
				'reupload_unclear_fork',
				'reupload_insufficient_fork',
				'reupload_request_proof',
				'reupload_identity_verification',
				'reupload_request_proof_server',
				'reupload_identity_verification_server',
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
			shouldShow: (project, projectV3) => !projectV3?.minecraft_server,
			message: async () => (await import('../messages/reupload/fork.md?raw')).default,
			disablesActions: [
				'reupload_reupload',
				'reupload_insufficient_fork',
				'reupload_request_proof',
				'reupload_identity_verification',
				'reupload_request_proof_server',
				'reupload_identity_verification_server',
			],
		} as ButtonAction,
		{
			id: 'reupload_insufficient_fork',
			type: 'button',
			label: 'Insufficient Fork',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			shouldShow: (project, projectV3) => !projectV3?.minecraft_server,
			message: async () => (await import('../messages/reupload/insufficient_fork.md?raw')).default,
			disablesActions: [
				'reupload_unclear_fork',
				'reupload_reupload',
				'reupload_request_proof',
				'reupload_identity_verification',
				'reupload_request_proof_server',
				'reupload_identity_verification_server',
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
				'reupload_request_proof_server',
				'reupload_identity_verification_server',
			],
		},
		{
			id: 'reupload_identity_verification',
			type: 'button',
			label: 'Verify Identity',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			shouldShow: (project, projectV3) => !projectV3?.minecraft_server,
			message: async () =>
				(await import('../messages/reupload/identity-verification/identity_verification.md?raw'))
					.default,
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
				'reupload_request_proof_server',
			],
		},
		{
			id: 'reupload_identity_verification_server',
			type: 'button',
			label: 'Verify Identity',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			shouldShow: (project, projectV3) => !!projectV3?.minecraft_server,
			message: async () =>
				(
					await import('../messages/reupload/identity-verification/identity_verification-server.md?raw')
				).default,
			relevantExtraInput: [
				{
					label: 'known public contact method',
					variable: 'CONTACT',
					required: true,
				},
			],
			disablesActions: [
				'reupload_reupload',
				'reupload_insufficient_fork',
				'reupload_request_proof',
				'reupload_request_proof_server',
			],
		},
		{
			id: 'reupload_request_proof_server',
			type: 'button',
			label: 'Reuploaded pack',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			shouldShow: (project, projectV3) =>
				!!projectV3?.minecraft_server &&
				projectV3?.minecraft_java_server?.content?.kind === 'modpack' &&
				projectV3?.minecraft_java_server?.content?.['project_id'] === project.id,
			message: async () =>
				(await import('../messages/reupload/custom_server/custom_server_permissions.md?raw'))
					.default,
			disablesActions: [
				'reupload_reupload',
				'reupload_unclear_fork',
				'reupload_insufficient_fork',
				'reupload_identity_verification',
				'reupload_request_proof',
			],
		},
		{
			id: 'reupload_custom_pack_verification',
			type: 'button',
			label: 'Override verification',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			shouldShow: (project, projectV3) =>
				!!projectV3?.minecraft_server &&
				projectV3?.minecraft_java_server?.content?.kind === 'modpack' &&
				projectV3?.minecraft_java_server?.content?.['project_id'] === project.id,
			message: async () =>
				(
					await import('../messages/reupload/custom_server/custom_server_overrides-verification.md?raw')
				).default,
			enablesActions: [
				{
					id: 'reupload_custom_pack_verification-list',
					type: 'toggle',
					label: 'List overrides?',
					weight: 1101,
					message: async () =>
						(
							await import('../messages/reupload/custom_server/custom_server_overrides-verification-list.md?raw')
						).default,
					relevantExtraInput: [
						{
							label: 'Add list of overrides.',
							variable: 'OVERRIDES',
							large: true,
							required: false,
						},
					],
				} as ToggleAction,
			],

			disablesActions: [
				'reupload_reupload',
				'reupload_unclear_fork',
				'reupload_insufficient_fork',
				'reupload_identity_verification',
				'reupload_request_proof',
				'reupload_custom_pack_prohibited',
			],
		},
		{
			id: 'reupload_custom_pack_prohibited',
			type: 'button',
			label: 'Forbidden Overrides',
			weight: 1100,
			suggestedStatus: 'rejected',
			severity: 'high',
			shouldShow: (project, projectV3) =>
				!!projectV3?.minecraft_server &&
				projectV3?.minecraft_java_server?.content?.kind === 'modpack' &&
				projectV3?.minecraft_java_server?.content?.['project_id'] === project.id,
			message: async () =>
				(
					await import('../messages/reupload/custom_server/custom_server_overrides-prohibited.md?raw')
				).default,
			relevantExtraInput: [
				{
					label: 'Add list of overrides.',
					variable: 'OVERRIDES',
					large: true,
					required: true,
				},
			],
			disablesActions: [
				'reupload_reupload',
				'reupload_unclear_fork',
				'reupload_insufficient_fork',
				'reupload_identity_verification',
				'reupload_request_proof',
				'reupload_custom_pack_verification',
			],
		},
	],
}

export default reupload
