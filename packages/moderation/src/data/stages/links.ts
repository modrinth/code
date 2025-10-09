import { LinkIcon } from '@modrinth/assets'

import type { ButtonAction } from '../../types/actions'
import type { Stage } from '../../types/stage'

const links: Stage = {
	title: "Are the project's links accurate and accessible?",
	id: 'links',
	icon: LinkIcon,
	guidance_url: 'https://modrinth.com/legal/rules',
	navigate: '/settings/links',
	shouldShow: (project) =>
		Boolean(
			project.issues_url ||
				project.source_url ||
				project.wiki_url ||
				project.discord_url ||
				project.donation_urls.length > 0,
		),
	text: async (project) => {
		let text = (await import('../messages/checklist-text/links/base.md?raw')).default

		if (project.donation_urls.length > 0) {
			text += (await import('../messages/checklist-text/links/donation/donations.md?raw')).default

			for (const donation of project.donation_urls) {
				text += (await import(`../messages/checklist-text/links/donation/donation.md?raw`)).default
					.replace('{URL}', donation.url)
					.replace('{PLATFORM}', donation.platform)
			}
		}

		return text
	},
	actions: [
		{
			id: 'links_misused',
			type: 'button',
			label: 'Links are misused',
			weight: 500,
			suggestedStatus: 'flagged',
			severity: 'low',
			message: async () => (await import('../messages/links/misused.md?raw')).default,
			relevantExtraInput: [
				{
					label: 'What links are misused?',
					variable: 'MISUSED_LINKS',
					required: false,
				},
			],
		} as ButtonAction,
		{
			id: 'links_inaccessible',
			type: 'button',
			label: 'Links are inaccessible',
			weight: 510,
			suggestedStatus: 'flagged',
			// Theoretically a conditional could go here to prevent overlap of misuse and inaccessible messages repeating while still allowing for a multi-select in each.
			// if links_misused was selected, send nothing.
			message: async () => (await import('../messages/links/not_accessible.md?raw')).default,
			enablesActions: [
				{
					id: 'links_inaccessible_options',
					type: 'multi-select-chips',
					label: 'Warn of inaccessible link?',
					shouldShow: (project) => Boolean(project.source_url || project.discord_url),
					options: [
						{
							label: 'Source',
							weight: 511,
							shouldShow: (project) => Boolean(project.source_url),
							message: async () =>
								(await import('../messages/links/not_accessible-source.md?raw')).default,
						},
						{
							label: 'Discord',
							weight: 512,
							shouldShow: (project) => Boolean(project.discord_url),
							message: async () =>
								(await import('../messages/links/not_accessible-discord.md?raw')).default,
						},
					],
				},
			],
		} as ButtonAction,
	],
}

export default links
