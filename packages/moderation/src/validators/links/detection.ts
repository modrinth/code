import LinkifyIt from 'linkify-it'

import { getBlockedProjectContentLink } from './syntax-checks.ts'

const linkify = new LinkifyIt({
	fuzzyEmail: false,
	fuzzyIP: true,
	fuzzyLink: true,
})

export function extractProjectLinks(text: string) {
	const matches = linkify.match(text) ?? []
	return [...new Set(matches.map((match) => match.url))]
}

export function containsProjectLinkOrIp(text: string) {
	return linkify.test(text)
}

export function containsExplicitHttpProjectLink(text: string) {
	return (linkify.match(text) ?? []).some((match) => {
		const schema = match.schema.toLowerCase()
		return schema === 'http:' || schema === 'https:'
	})
}

export function findBlockedProjectContentLink(text: string) {
	for (const url of extractProjectLinks(text)) {
		const blockedLink = getBlockedProjectContentLink(url)
		if (blockedLink) return blockedLink
	}

	return null
}
