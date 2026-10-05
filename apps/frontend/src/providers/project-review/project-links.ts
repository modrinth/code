import type { Labrinth } from '@modrinth/api-client'

export function previousReviewLinkUrls(issues: readonly Labrinth.Threads.v3.ThreadIssue[]) {
	const links = new Map<string, string>()
	for (const issue of issues) {
		if (issue.verdict === 'resolved') continue
		for (const { what, verdict } of issue.facets) {
			if (what.type !== 'modify_links' || verdict === 'resolved') continue
			for (const [key, link] of Object.entries(what.value.links)) {
				links.set(key, link.original)
			}
		}
	}
	return links
}

export function reviewExternalUrl(value: string | undefined): string | undefined {
	if (!value) return
	try {
		const url = new URL(value)
		if (url.protocol === 'https:' || url.protocol === 'http:') return url.href
	} catch {
		return undefined
	}
}
