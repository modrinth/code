import { LibraryIcon } from '@modrinth/assets'

import type { ButtonAction, MultiSelectChipsAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const description: Stage = {
	title: 'Is the description sufficient, accurate, and accessible?',
	id: 'description',
	icon: LibraryIcon,
	guidance_url:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080508042e70089dd787e',
	navigate: '/',
	actions: [
		{
			id: 'description_insufficient',
			type: 'button',
			label: 'Insufficient',
			weight: 400,
			suggestedStatus: 'flagged',
			severity: 'medium',
			message: async () =>
				(await import('../messages/description/insufficient/insufficient-header.md?raw')).default,
			enablesActions: [
				{
					id: 'description_insufficient_options',
					type: 'multi-select-chips',
					label: 'Which details are missing from the description?',
					options: [
						{
							id: 'description_insufficient_packs',
							label: 'Missing basic details (modpacks)',
							weight: 401,
							shouldShow: (project, projectV3) =>
								project.project_type === 'modpack' && !projectV3?.minecraft_server,
							message: async () =>
								(await import('../messages/checklist-messages/description/insufficient/insufficient-packs.md?raw'))
									.default,
						},
						{
							id: 'description_insufficient_projects',
							label: 'Missing basic details',
							weight: 402,
							shouldShow: (project, projectV3) =>
								project.project_type !== 'modpack' && !projectV3?.minecraft_server,
							message: async () =>(
								await import('../messages/checklist-messages/description/insufficient/insufficient-projects.md?raw'))
									.default,
						},
						{
							id: 'description_insufficient_servers',
							label: 'Missing basic details',
							weight: 403,
							shouldShow: (project, projectV3) => !!projectV3?.minecraft_java_server,
							message: async () =>(
								await import('../messages/checklist-messages/description/insufficient/insufficient-servers.md?raw'))
									.default,
						},
						{
							id: 'description_insufficient_fork',
							label: 'Fork',
							weight: 404,
							message: async () =>
								(await import('../messages/description/insufficient/insufficient-fork.md?raw'))
									.default,
						},
						{
							id: 'description_insufficient_port',
							label: 'Port',
							weight: 405,
							message: async () =>
								(await import('../messages/description/insufficient/insufficient-port.md?raw'))
									.default,
						},
					],
				} as MultiSelectChipsAction,
				{
					id: 'description_insufficient_custom',
					type: 'button',
					label: 'Insufficient (custom)',
					weight: 406,
					suggestedStatus: 'flagged',
					severity: 'medium',
					message: async () =>(
						await import('../messages/checklist-messages/description/insufficient/insufficient.md?raw')).default,
					relevantExtraInput: [
						{
							label: 'Please elaborate on how the author can improve their description.',
							variable: 'EXPLAINER',
							large: true,
							required: true,
						},
					],
				} as ButtonAction,
			],
		} as ButtonAction,
		{
			id: 'description_non_english',
			type: 'button',
			label: 'Non-english',
			weight: 402,
			suggestedStatus: 'flagged',
			severity: 'medium',
			shouldShow: (project, projectV3) => !projectV3?.minecraft_java_server,
			message: async () =>
				(
					await import('../messages/checklist-messages/description/accessability/non-english/non-english.md?raw')
				).default,
		} as ButtonAction,
		{
			id: 'description_non_english-server',
			type: 'button',
			label: 'Non-english',
			weight: 402,
			suggestedStatus: 'flagged',
			severity: 'medium',
			shouldShow: (project, projectV3) => !!projectV3?.minecraft_java_server,
			message: async () =>
				(
					await import('../messages/checklist-messages/description/accessability/non-english/non-english-server.md?raw')
				).default,
		} as ButtonAction,
		{
			id: 'description_unfinished',
			type: 'button',
			label: 'Unfinished',
			weight: 403,
			suggestedStatus: 'flagged',
			severity: 'low',
			message: async () =>
				(await import('../messages/checklist-messages/description/unfinished.md?raw')).default,
		} as ButtonAction,
		{
			id: 'description_headers_as_body',
			type: 'button',
			label: 'Headers as body text',
			weight: 404,
			suggestedStatus: 'flagged',
			severity: 'low',
			message: async () =>
				(
					await import('../messages/checklist-messages/description/accessability/headers-as-body.md?raw')
				).default,
		} as ButtonAction,
		{
			id: 'description_image_only',
			type: 'button',
			label: 'Image-only',
			weight: 405,
			suggestedStatus: 'flagged',
			severity: 'medium',
			message: async () =>
				(await import('../messages/checklist-messages/description/accessability/image-only.md?raw'))
					.default,
		} as ButtonAction,
		{
			id: 'description_non_standard_text',
			type: 'button',
			label: 'Non-standard text',
			weight: 406,
			suggestedStatus: 'flagged',
			severity: 'medium',
			message: async () =>
				(
					await import('../messages/checklist-messages/description/accessability/non-standard-text.md?raw')
				).default,
		} as ButtonAction,
		{
			id: 'description_clarity',
			type: 'button',
			label: 'Unclear / Misleading',
			weight: 407,
			suggestedStatus: 'rejected',
			severity: 'high',
			message: async () =>
				(await import('../messages/checklist-messages/description/clarity.md?raw')).default,
		} as ButtonAction,
	],
}

export default description
