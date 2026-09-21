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
import { issue, panel, section, toggle } from './component-builders/builders'

const linkProblemLabels = {
	misused: 'Misused',
	inaccessible: 'Inaccessible',
	disabled: 'Disabled',
	empty: 'Empty Repo',
	expiring: 'Expiring',
} as const

type LinkProblem = keyof typeof linkProblemLabels

const linkFields = {
	'source-link': { label: 'Source code', problems: ['misused', 'inaccessible', 'empty'] },
	'discord-link': { label: 'Discord invite', problems: ['misused', 'inaccessible', 'expiring'] },
	'issues-link': { label: 'Issue tracker', problems: ['misused', 'inaccessible', 'disabled'] },
	'wiki-link': { label: 'Wiki page', problems: ['misused', 'inaccessible', 'disabled'] },
	'site-link': { label: 'Website', problems: ['misused', 'inaccessible'] },
	'store-link': { label: 'Store', problems: ['misused', 'inaccessible'] },
	'patreon-link': { label: 'Patreon', problems: ['misused', 'inaccessible'] },
	'bmac-link': { label: 'Buy Me A Coffee', problems: ['misused', 'inaccessible'] },
	'paypal-link': { label: 'PayPal', problems: ['misused', 'inaccessible'] },
	'github-link': { label: 'GitHub Sponsors', problems: ['misused', 'inaccessible'] },
	'ko-fi-link': { label: 'Ko-fi', problems: ['misused', 'inaccessible'] },
	'other-link': { label: 'Other', problems: ['misused', 'inaccessible'] },
} as const satisfies Record<string, { label: string; problems: readonly LinkProblem[] }>

type LinkField = keyof typeof linkFields

const linkProblemGroups: {
	header: string
	problems: { key: LinkProblem; notes?: Partial<Record<LinkField, string>> }[]
}[] = [
	{
		header: inaccessibleHeader,
		problems: [
			{
				key: 'inaccessible',
				notes: {
					'source-link': sourceInaccessible,
					'discord-link': discordInaccessible,
				},
			},
			{
				key: 'disabled',
				notes: { 'issues-link': issuesDisabled, 'wiki-link': wikiDisabled },
			},
			{ key: 'expiring', notes: { 'discord-link': discordExpiring } },
		],
	},
	{
		header: misusedHeader,
		problems: [{ key: 'misused' }, { key: 'empty', notes: { 'source-link': sourceEmpty } }],
	},
]

export const linksIssue = issue({
	id: 'links',
	suggestedStatus: 'flagged',
	message: ({ ProjectV3, selected }) => {
		const toggleIds = new Set(selected.toggleIds)
		const groups = linkProblemGroups.flatMap(({ header, problems }) => {
			const items = Object.entries(linkFields).flatMap(([field, { label }]) => {
				const selectedProblems = problems.filter(({ key }) => toggleIds.has(`${field}:${key}`))
				if (!selectedProblems.length) return []
				const url = ProjectV3.link_urls[field.replace(/-link$/, '')]?.url
				const notes = selectedProblems
					.map(({ notes }) => notes?.[field as LinkField]?.trim())
					.filter((note): note is string => !!note)
				const item = url ? `- ${label}: \`${url}\`` : `- ${label}`
				return [[item, ...new Set(notes)].join('\n    - ')]
			})
			return items.length ? [header.trim(), items.join('\n')] : []
		})
		return groups.length ? [linksHeader.trim(), ...groups].join('\n\n') : ''
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

function createLinkToggle(field: LinkField, problem: LinkProblem) {
	return toggle({
		label: linkProblemLabels[problem],
		id: `${field}:${problem}`,
		issue: linksIssue,
	})
}

function createLinkPanel(field: LinkField) {
	const { label, problems } = linkFields[field]
	const linkType = field.replace(/-link$/, '')

	return panel({
		field,
		title: label,
		hint: "Is the project's link accurate and accessible?",
		icon: LinkIcon,
		shown: ({ ProjectV3 }) => isHttpUrl(ProjectV3.link_urls[linkType]?.url),
	}).content(section().content(...problems.map((problem) => createLinkToggle(field, problem))))
}

export const issuesReviewPanel = createLinkPanel('issues-link')
export const sourceReviewPanel = createLinkPanel('source-link')
export const wikiReviewPanel = createLinkPanel('wiki-link')
export const discordReviewPanel = createLinkPanel('discord-link')
export const siteReviewPanel = createLinkPanel('site-link')
export const storeReviewPanel = createLinkPanel('store-link')
export const patreonReviewPanel = createLinkPanel('patreon-link')
export const bmacReviewPanel = createLinkPanel('bmac-link')
export const paypalReviewPanel = createLinkPanel('paypal-link')
export const githubReviewPanel = createLinkPanel('github-link')
export const koFiReviewPanel = createLinkPanel('ko-fi-link')
export const otherReviewPanel = createLinkPanel('other-link')
