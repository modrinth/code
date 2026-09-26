import { LinkIcon } from '@modrinth/assets'

import discordExpiring from '../messages/checklist/messages/links/discord/expiring.md'
import discordInaccessible from '../messages/checklist/messages/links/discord/inaccessible.md'
import linksHeader from '../messages/checklist/messages/links/header.md'
import inaccessibleHeader from '../messages/checklist/messages/links/inaccessible-header.md'
import issuesDisabled from '../messages/checklist/messages/links/issues/disabled.md'
import misusedHeader from '../messages/checklist/messages/links/misused-header.md'
import sourceEmpty from '../messages/checklist/messages/links/source/empty.md'
import sourceInaccessible from '../messages/checklist/messages/links/source/inaccessible.md'
import wikiDisabled from '../messages/checklist/messages/links/wiki/disabled.md'
import { issue, panel, toggle } from './component-builders/builders'

export const linksIssue = issue({
	id: 'links',
	title: 'Invalid project links',
	category: 'Links',
	suggestedStatus: 'flagged',
	message: ({ ProjectV3, selected }) => {
		const toggleIds = new Set(selected.toggleIds)
		const inaccessibleItems: string[] = []
		const misusedItems: string[] = []

		for (const type of [
			'source',
			'discord',
			'issues',
			'wiki',
			'site',
			'store',
			'patreon',
			'bmac',
			'paypal',
			'github',
			'ko-fi',
			'other',
		] as const) {
			const field = `${type}-link`
			let inaccessibleSelected = toggleIds.has(`${field}:inaccessible`)
			const misusedSelected = toggleIds.has(`${field}:misused`)
			const notes: string[] = []
			let label: string

			switch (type) {
				case 'source':
					label = 'Source code'
					if (inaccessibleSelected) notes.push(sourceInaccessible)
					if (toggleIds.has('source-link:empty')) {
						inaccessibleSelected = true
						notes.push(sourceEmpty)
					}
					break
				case 'discord':
					label = 'Discord invite'
					if (inaccessibleSelected) notes.push(discordInaccessible)
					if (toggleIds.has('discord-link:expiring')) {
						inaccessibleSelected = true
						notes.push(discordExpiring)
					}
					break
				case 'issues':
					label = 'Issue tracker'
					if (toggleIds.has('issues-link:disabled')) {
						inaccessibleSelected = true
						notes.push(issuesDisabled)
					}
					break
				case 'wiki':
					label = 'Wiki page'
					if (toggleIds.has('wiki-link:disabled')) {
						inaccessibleSelected = true
						notes.push(wikiDisabled)
					}
					break
				case 'site':
					label = 'Website'
					break
				case 'store':
					label = 'Store'
					break
				case 'patreon':
					label = 'Patreon'
					break
				case 'bmac':
					label = 'Buy Me A Coffee'
					break
				case 'paypal':
					label = 'PayPal'
					break
				case 'github':
					label = 'GitHub Sponsors'
					break
				case 'ko-fi':
					label = 'Ko-fi'
					break
				case 'other':
					label = 'Other'
					break
				default:
					continue
			}

			if (!inaccessibleSelected && !misusedSelected) continue
			const url = ProjectV3.link_urls[type]?.url
			// "- Source code: `https://github.com/owner/repo`" or "- Source code"
			const item = url ? `- ${label}: \`${url}\`` : `- ${label}`
			if (inaccessibleSelected) {
				const uniqueNotes = [...new Set(notes.map((note) => note.trim()).filter(Boolean))]
				inaccessibleItems.push([item, ...uniqueNotes].join('\n    - '))
			}
			if (misusedSelected) misusedItems.push(item)
		}

		const sections: string[] = []
		if (inaccessibleItems.length) {
			sections.push(inaccessibleHeader.trim(), inaccessibleItems.join('\n'))
		}
		if (misusedItems.length) {
			sections.push(misusedHeader.trim(), misusedItems.join('\n'))
		}
		return sections.length ? [linksHeader.trim(), ...sections].join('\n\n') : ''
	},
})

function isHttpUrl(value: string | undefined): boolean {
	if (!value) return false
	try {
		const url = new URL(value)
		return url.protocol === 'https:' || url.protocol === 'http:'
	} catch {
		return false
	}
}

export const issuesReviewPanel = panel({
	title: 'Issue tracker',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.issues?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'issues-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'issues-link:inaccessible',
		label: 'Inaccessible',
	}),
	toggle({
		issue: linksIssue,
		id: 'issues-link:disabled',
		label: 'Disabled',
		issueListGroup: 'Inaccessible',
		issueListLabel: 'Issues: Disabled',
	}),
)

export const sourceReviewPanel = panel({
	title: 'Source code',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.source?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'source-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'source-link:inaccessible',
		label: 'Inaccessible',
	}),
	toggle({
		issue: linksIssue,
		id: 'source-link:empty',
		label: 'Empty Repo',
		issueListGroup: 'Inaccessible',
		issueListLabel: 'Source: Empty Repo',
	}),
)

export const wikiReviewPanel = panel({
	title: 'Wiki page',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.wiki?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'wiki-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'wiki-link:inaccessible',
		label: 'Inaccessible',
	}),
	toggle({
		issue: linksIssue,
		id: 'wiki-link:disabled',
		label: 'Disabled',
		issueListGroup: 'Inaccessible',
		issueListLabel: 'Wiki: Disabled',
	}),
)

export const discordReviewPanel = panel({
	title: 'Discord invite',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.discord?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'discord-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'discord-link:inaccessible',
		label: 'Inaccessible',
	}),
	toggle({
		issue: linksIssue,
		id: 'discord-link:expiring',
		label: 'Expiring',
		issueListGroup: 'Inaccessible',
		issueListLabel: 'Discord: Expiring',
	}),
)

export const siteReviewPanel = panel({
	title: 'Website',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.site?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'site-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'site-link:inaccessible',
		label: 'Inaccessible',
	}),
)

export const storeReviewPanel = panel({
	title: 'Store',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.store?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'store-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'store-link:inaccessible',
		label: 'Inaccessible',
	}),
)

export const patreonReviewPanel = panel({
	title: 'Patreon',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.patreon?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'patreon-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'patreon-link:inaccessible',
		label: 'Inaccessible',
	}),
)

export const bmacReviewPanel = panel({
	title: 'Buy Me A Coffee',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.bmac?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'bmac-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'bmac-link:inaccessible',
		label: 'Inaccessible',
	}),
)

export const paypalReviewPanel = panel({
	title: 'PayPal',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.paypal?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'paypal-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'paypal-link:inaccessible',
		label: 'Inaccessible',
	}),
)

export const githubReviewPanel = panel({
	title: 'GitHub Sponsors',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.github?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'github-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'github-link:inaccessible',
		label: 'Inaccessible',
	}),
)

export const koFiReviewPanel = panel({
	title: 'Ko-fi',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls['ko-fi']?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'ko-fi-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'ko-fi-link:inaccessible',
		label: 'Inaccessible',
	}),
)

export const otherReviewPanel = panel({
	title: 'Other',
	hint: "Is the project's link accurate and accessible?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf08013b36cd75cbf1a9177',
	shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls.other?.url),
}).content(
	toggle({
		issue: linksIssue,
		id: 'other-link:misused',
		label: 'Misused',
	}),
	toggle({
		issue: linksIssue,
		id: 'other-link:inaccessible',
		label: 'Inaccessible',
	}),
)
