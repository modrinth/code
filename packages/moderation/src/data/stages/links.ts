import { LinkIcon } from '@modrinth/assets'

import type {
	ButtonAction,
	ChecklistActionContext,
	MultiSelectChipsAction,
	MultiSelectChipsOption,
} from '../../types/actions'
import type { Stage } from '../../types/stage'

interface LinkOptionDefinition {
	id: string
	label: string
	weight: number
	message: () => Promise<string>
}

function formatLabel(value: string): string {
	return value
		.split(/[-_]/g)
		.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
		.join(' ')
}

function getProjectLinkDefinitions(context: ChecklistActionContext): LinkOptionDefinition[] {
	const { project, projectV3 } = context
	const donationUrls = project.donation_urls ?? []
	const links: LinkOptionDefinition[] = []
	let weight = 511

	const pushLink = (id: string, label: string, message: () => Promise<string>) => {
		links.push({ id, label, weight, message })
		weight += 1
	}

	if (project.issues_url) {
		pushLink(
			'issues',
			'Issues',
			async () =>
				'Currently, your Issues link is inaccessible. Make sure it points to a publicly accessible issue tracker before resubmitting your project.',
		)
	}

	if (project.source_url) {
		pushLink(
			'source',
			'Source',
			async () => (await import('../messages/links/not_accessible-source.md?raw')).default,
		)
	}

	if (project.wiki_url) {
		pushLink(
			'wiki',
			'Wiki',
			async () =>
				'Currently, your Wiki link is inaccessible. Make sure it points to a publicly accessible documentation page before resubmitting your project.',
		)
	}

	if (project.discord_url) {
		pushLink(
			'discord',
			'Discord',
			async () => (await import('../messages/links/not_accessible-discord.md?raw')).default,
		)
	}

	if (projectV3?.link_urls?.site?.url) {
		pushLink(
			'site',
			'Website',
			async () =>
				'Currently, your Website link is inaccessible. Make sure it points to a publicly accessible project website before resubmitting your project.',
		)
	}

	if (projectV3?.link_urls?.store?.url) {
		pushLink(
			'store',
			'Store',
			async () =>
				'Currently, your Store link is inaccessible. Make sure it points to a publicly accessible storefront before resubmitting your project.',
		)
	}

	donationUrls.forEach((donation, index) => {
		const donationLabel = `${formatLabel(donation.platform)} Donation`
		pushLink(
			`donation-${donation.id}-${index}`,
			donationLabel,
			async () =>
				`Currently, your ${donationLabel} link is inaccessible. Make sure it points to a publicly accessible donation page before resubmitting your project.`,
		)
	})

	return links
}

function getMisusedLinkOptions(context: ChecklistActionContext): MultiSelectChipsOption[] {
	return getProjectLinkDefinitions(context).map((link, index) => ({
		id: link.id,
		label: link.label,
		weight: 501 + index,
		message: async () => `- ${link.label}`,
	}))
}

function getInaccessibleLinkOptions(context: ChecklistActionContext): MultiSelectChipsOption[] {
	return getProjectLinkDefinitions(context).map((link) => ({
		id: link.id,
		label: link.label,
		weight: link.weight,
		message: link.message,
	}))
}

const links: Stage = {
	title: "Are the project's links accurate and accessible?",
	id: 'links',
	icon: LinkIcon,
	guidance_url:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	navigate: '/settings/links',
	shouldShow: (project, projectV3) =>
		Boolean(
			project.issues_url ||
			project.source_url ||
			project.wiki_url ||
			project.discord_url ||
			projectV3?.link_urls?.site ||
			projectV3?.link_urls?.store ||
			(project.donation_urls?.length ?? 0) > 0,
		),
	text: async (project, projectV3) => {
		let text
		if (projectV3?.minecraft_server)
			text = (await import('../messages/checklist-text/links/server.md?raw')).default
		else text = (await import('../messages/checklist-text/links/base.md?raw')).default

		if ((project.donation_urls?.length ?? 0) > 0) {
			text += (await import('../messages/checklist-text/links/donation/donations.md?raw')).default

			for (const donation of project.donation_urls ?? []) {
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
			message: async () =>
				(await import('../messages/checklist-messages/links/misused.md?raw')).default,
			enablesActions: [
				{
					id: 'links_misused_options',
					type: 'multi-select-chips',
					label: 'Which links are misused?',
					joinWith: '\n',
					shouldShow: (_project, _projectV3, context) =>
						Boolean(context && getMisusedLinkOptions(context).length > 0),
					options: (context) => getMisusedLinkOptions(context),
				},
			] as MultiSelectChipsAction[],
		} as ButtonAction,
		{
			id: 'links_inaccessible',
			type: 'button',
			label: 'Links are inaccessible',
			weight: 510,
			suggestedStatus: 'flagged',
			// Theoretically a conditional could go here to prevent overlap of misuse and inaccessible messages repeating while still allowing for a multi-select in each.
			// if links_misused was selected, send nothing.
			message: async () =>
				(await import('../messages/checklist-messages/links/not_accessible.md?raw')).default,
			enablesActions: [
				{
					id: 'links_inaccessible_options',
					type: 'multi-select-chips',
					label: 'Warn of inaccessible link?',
					shouldShow: (_project, _projectV3, context) =>
						Boolean(context && getInaccessibleLinkOptions(context).length > 0),
					options: (context) => getInaccessibleLinkOptions(context),
				} as MultiSelectChipsAction,
			],
		} as ButtonAction,
	],
}

export default links
