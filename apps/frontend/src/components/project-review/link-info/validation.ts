import type { Labrinth } from '@modrinth/api-client'
import { normalizeProjectNagKind } from '@modrinth/moderation'

export type LinkValidationWarning = { code: string; text: string }

const fieldNames: Record<string, string> = {
	issues: 'issue tracker',
	source: 'source code',
	wiki: 'wiki',
	discord: 'Discord invite',
	site: 'website',
	store: 'store',
	license: 'license',
	description: 'description',
	patreon: 'Patreon',
	bmac: 'Buy Me a Coffee',
	paypal: 'PayPal',
	github: 'GitHub Sponsors',
	kofi: 'Ko-fi',
}

function warningText(details: Labrinth.Projects.v3.ProjectNag['details']) {
	const rawField = typeof details.field === 'string' ? details.field : 'other'
	const field = rawField === 'ko-fi' ? 'kofi' : rawField
	const name = fieldNames[field === 'ko-fi' ? 'kofi' : field] ?? 'donation'
	const otherField = typeof details.other_field === 'string' ? details.other_field : ''
	const otherName =
		otherField === 'description'
			? undefined
			: fieldNames[otherField === 'ko-fi' ? 'kofi' : otherField]
	const other = otherName ? otherName[0].toUpperCase() + otherName.slice(1) : 'another field'
	switch (details.reason) {
		case 'global_blocklist_match':
			return `Your ${name} link uses a prohibited domain. Remove or replace this link.`
		case 'external_blocklist_match':
			return `Your ${field} link is not allowed. Please remove this link, or replace it with something appropriate for the ${field} link type.`
		case 'wrong_field':
			return `Your ${name} link is not correct for ${name} links. Remove or replace it with the correct link.`
		case 'ip_address':
			return `Your ${name} link uses an IP address. Use a URL with a domain name.`
		case 'malformed':
			return `Your ${name} link is invalid. Enter a complete HTTPS URL.`
		case 'not_in_allowlist':
			return `Your ${name} link may not be appropriate for the link type. Please ensure that it is suitable for its intended purpose.`
		case 'duplicate':
			return `Your ${name} link is also used in ${other}. Each link should be listed only once.`
		case 'download':
			return 'Links that start a download are not allowed. Please remove any such links from your description.'
		case 'discord_invite':
			return 'Your Discord invite is invalid, expired, or does not invite users to a server. Replace it with an active server invite.'
		case 'source_repository':
			return 'Your source code link must lead to a repository on a supported host platform, or to a self-hosted Gitea/Forgejo service.'
		default:
			return `Your ${name} link could not be verified. Check that it is online and accessible.`
	}
}

export function linkValidationWarnings(
	nags: readonly Labrinth.Projects.v3.ProjectNag[],
	field: string,
	url: string,
): LinkValidationWarning[] {
	return nags.flatMap((nag) => {
		if (nag.severity === 'suggestion' || normalizeProjectNagKind(nag.kind) !== 'link-validation')
			return []
		const details = nag.details
		if (typeof details.field === 'string' && details.field !== field) return []
		if (Array.isArray(details.fields) && !details.fields.includes(field)) return []
		if (typeof details.gallery_index === 'number' || typeof details.url !== 'string') return []
		try {
			if (new URL(details.url).href !== new URL(url).href) return []
		} catch {
			return []
		}
		return [
			{
				code: `link-validation:${JSON.stringify([details.field, details.url, details.reason])}`,
				text: warningText(details),
			},
		]
	})
}
