import { EXTERNAL_LINKS_BLOCK_LIST, URL_SHORTENERS } from './block-list.ts'
import { PROJECT_LINK_DOMAIN_LIST } from './domain-list.ts'
import type {
	BlockedProjectLink,
	FieldMatcher,
	LinkCheckBuilder,
	LinkCheckChildShape,
	LinkCheckContext,
	LinkCheckMatcher,
	LinkCheckNode,
	LinkCheckResult,
	LinkCheckVerify,
	MatchResult,
	MessageDescriptor,
	RemoteLinkCheckVerify,
} from './types.ts'

function warn(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckResult {
	return { severity: 'warn', message, values }
}

function error(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckResult {
	return { severity: 'error', message, values }
}

export function matchesField(matcher: FieldMatcher, context: LinkCheckContext): boolean {
	if (typeof matcher === 'function') return matcher(context.field, context)
	if (Array.isArray(matcher)) return matcher.includes(context.field)
	return matcher === context.field
}

function isAsyncMatcher(when: LinkCheckMatcher): boolean {
	return when instanceof RegExp ? false : when.constructor.name === 'AsyncFunction'
}

function buildNode(when: LinkCheckMatcher, label?: string): LinkCheckBuilder {
	const childNodes: LinkCheckNode[] = []
	const forMatchers: FieldMatcher[] = []
	const node: Record<string, unknown> = { when, label, childNodes, forMatchers }

	node.for = (fields: FieldMatcher) => {
		forMatchers.push(fields)
		return node
	}
	node.verify = (fn: LinkCheckVerify) => {
		node.verifyMatch = fn
		return node
	}
	node.verifyRemotely = (fn: RemoteLinkCheckVerify) => {
		node.verifyMatch = fn
		node.isRemoteVerification = true
		return node
	}
	node.severity = (value: 'error' | 'warn') => {
		node.unrecognizedSeverity = value
		return node
	}
	node.message = (descriptor: MessageDescriptor) => {
		node.unrecognizedMessage = descriptor
		return node
	}
	node.transparent = () => {
		node.isTransparent = true
		return node
	}
	node.fallback = () => {
		node.isFallback = true
		return node
	}
	node.warn = (message: MessageDescriptor, values?: Record<string, unknown>) => {
		node.verifyMatch = async () => warn(message, values)
		return node
	}
	node.error = (message: MessageDescriptor, values?: Record<string, unknown>) => {
		node.verifyMatch = async () => error(message, values)
		return node
	}
	node.children = (...shapes: LinkCheckChildShape[]) => {
		const parentLabel = node.label as string | undefined
		const parentForMatchers = node.forMatchers as FieldMatcher[] | undefined
		for (const shape of shapes) {
			const child = normalizeChild(shape)
			const label = child.label ? [parentLabel, child.label].filter(Boolean).join(' ') : parentLabel
			const inheritedFor = [...(parentForMatchers ?? []), ...(child.forMatchers ?? [])]
			childNodes.push({ ...child, label, forMatchers: inheritedFor })
		}
		return node
	}

	return node as unknown as LinkCheckBuilder
}

export function check(
	when: RegExp | string | ((remaining: string) => number | null | Promise<number | null>),
	label?: string,
): LinkCheckBuilder {
	const matcher =
		typeof when === 'function' ? when : typeof when === 'string' ? new RegExp(when) : when
	return buildNode(matcher, label)
}

export function fallback(label?: string): LinkCheckBuilder {
	return buildNode(() => 0, label).fallback()
}

function normalizeChild(shape: LinkCheckChildShape): LinkCheckNode {
	if (shape instanceof RegExp || typeof shape === 'function') return { when: shape }
	if (typeof shape === 'string') return { when: new RegExp(shape) }
	return shape as unknown as LinkCheckNode
}

export function named(label: string, shapes: LinkCheckChildShape[]): LinkCheckNode[] {
	return shapes.map((shape) => ({ ...normalizeChild(shape), label }))
}

export function anchored(source: string): RegExp {
	return new RegExp(`^${source}`, 'i')
}

export async function matchNode(
	node: LinkCheckNode,
	remaining: string,
	context: LinkCheckContext,
	isRoot = false,
): Promise<MatchResult | null> {
	let match: RegExpMatchArray | null
	if (node.when instanceof RegExp) {
		match = node.when.exec(remaining)
	} else {
		const consumed = await node.when(remaining)
		match =
			consumed === null
				? null
				: (Object.assign([remaining.slice(0, consumed)], {
						input: remaining,
						index: 0,
					}) as RegExpMatchArray)
	}

	if (!match) {
		if (!isRoot || !node.unrecognizedMessage) return null
		return {
			node: {
				when: node.when,
				label: node.label,
				unrecognizedMessage: node.unrecognizedMessage,
				unrecognizedSeverity: node.unrecognizedSeverity,
			},
			match: Object.assign([remaining], { input: remaining, index: 0 }) as RegExpMatchArray,
		}
	}

	if (node.childNodes?.length) {
		const rest = remaining.slice(match[0].length)
		const syncChildren = node.childNodes.filter(
			(child) => !isAsyncMatcher(child.when) && !child.isFallback,
		)
		const asyncChildren = node.childNodes.filter(
			(child) =>
				isAsyncMatcher(child.when) &&
				!child.isFallback &&
				!(context.generalContent && hasFieldSpecificDescendant(child)),
		)
		const fallbackChildren = node.childNodes.filter((child) => child.isFallback)
		let expectedChild: LinkCheckNode | undefined
		for (const child of [...syncChildren, ...asyncChildren]) {
			const found = await matchNode(child, rest, context)
			if (found) return found
			if (!expectedChild && child.forMatchers?.some((matcher) => matchesField(matcher, context)))
				expectedChild = child
		}

		const matchingFallback = fallbackChildren.find((child) =>
			child.forMatchers?.some((matcher) => matchesField(matcher, context)),
		)
		if (matchingFallback) {
			return {
				node: matchingFallback,
				match: Object.assign([rest], { input: rest, index: 0 }) as RegExpMatchArray,
			}
		}

		if (node.isTransparent) return null
		return { node, match, expectedChild }
	}

	return { node, match }
}

export function matchNodeSyntax(
	node: LinkCheckNode,
	remaining: string,
	context: LinkCheckContext,
	isRoot = false,
): MatchResult | null {
	let match: RegExpMatchArray | null
	if (node.when instanceof RegExp) {
		match = node.when.exec(remaining)
	} else {
		if (isAsyncMatcher(node.when)) return null
		const consumed = node.when(remaining)
		if (consumed instanceof Promise) return null
		match =
			consumed === null
				? null
				: (Object.assign([remaining.slice(0, consumed)], {
						input: remaining,
						index: 0,
					}) as RegExpMatchArray)
	}

	if (!match) {
		if (!isRoot || !node.unrecognizedMessage) return null
		return {
			node: {
				when: node.when,
				label: node.label,
				unrecognizedMessage: node.unrecognizedMessage,
				unrecognizedSeverity: node.unrecognizedSeverity,
			},
			match: Object.assign([remaining], { input: remaining, index: 0 }) as RegExpMatchArray,
		}
	}

	if (node.childNodes?.length) {
		const rest = remaining.slice(match[0].length)
		const children = node.childNodes.filter(
			(child) => !isAsyncMatcher(child.when) && !child.isFallback,
		)
		const fallbackChildren = node.childNodes.filter((child) => child.isFallback)
		let expectedChild: LinkCheckNode | undefined
		for (const child of children) {
			const found = matchNodeSyntax(child, rest, context)
			if (found) return found
			if (!expectedChild && child.forMatchers?.some((matcher) => matchesField(matcher, context)))
				expectedChild = child
		}

		const matchingFallback = fallbackChildren.find((child) =>
			child.forMatchers?.some((matcher) => matchesField(matcher, context)),
		)
		if (matchingFallback) {
			return {
				node: matchingFallback,
				match: Object.assign([rest], { input: rest, index: 0 }) as RegExpMatchArray,
			}
		}

		if (node.isTransparent) return null
		return { node, match, expectedChild }
	}

	return { node, match }
}

export function validUrlPrefix(remaining: string): number | null {
	let url: URL
	try {
		url = new URL(remaining)
		const hostname = url.hostname

		if (url.protocol !== 'https:') return null
		if (!/[^.]\.[^.]/.test(hostname)) return null
		if (/(^|\.)(local|localhost|test|example|invalid|onion|arpa|home)$/i.test(hostname)) return null
		if (/^example\.(com|net|org)$/i.test(hostname)) return null

		const strippedHost = hostname.replace(/^\[|]$/g, '')
		if (/^\d{1,3}(?:\.\d{1,3}){3}$/.test(strippedHost) || strippedHost.includes(':')) return null

		const protocolPrefix = /^https:\/\//i.exec(remaining)
		return protocolPrefix?.[0].length ?? null
	} catch {
		return null
	}
}

export function getLinkHostname(url: string | null | undefined): string | null {
	if (!url) return null
	try {
		const hostname = new URL(url).hostname.toLowerCase().replace(/\.$/, '')
		return hostname || null
	} catch {
		return null
	}
}

export function hostnameMatchesDomain(hostname: string, domain: string): boolean {
	return hostname === domain || hostname.endsWith(`.${domain}`)
}

function isLinkFromDomains(url: string | null | undefined, domains: readonly string[]): boolean {
	const hostname = getLinkHostname(url)
	return hostname !== null && domains.some((domain) => hostnameMatchesDomain(hostname, domain))
}

export function isCommonProjectLink(
	url: string | null | undefined,
	field: keyof typeof PROJECT_LINK_DOMAIN_LIST.common,
): boolean {
	return isLinkFromDomains(url, PROJECT_LINK_DOMAIN_LIST.common[field])
}

export function isDiscordLink(url: string | null | undefined): boolean {
	return isCommonProjectLink(url, 'discord')
}

export function isInappropriateLicenseLink(url: string | null | undefined): boolean {
	return isLinkFromDomains(url, PROJECT_LINK_DOMAIN_LIST.inappropriateLicense)
}

export function hasFieldSpecificDescendant(node: LinkCheckNode): boolean {
	return (
		(node.forMatchers?.length ?? 0) > 0 ||
		(node.childNodes?.some((child) => hasFieldSpecificDescendant(child)) ?? false)
	)
}

export function isIpAddress(hostname: string): boolean {
	const strippedHostname = hostname.replace(/^\[|]$/g, '')
	return /^\d{1,3}(?:\.\d{1,3}){3}$/.test(strippedHostname) || strippedHostname.includes(':')
}

export function getBlockedProjectExternalLink(url: string): BlockedProjectLink | null {
	const hostname = getLinkHostname(url)
	if (!hostname) return null

	if (isIpAddress(hostname)) return { label: 'IP address', url }

	if (URL_SHORTENERS.some((domain) => hostnameMatchesDomain(hostname, domain))) {
		return { label: 'URL shortener', url }
	}

	const entry = EXTERNAL_LINKS_BLOCK_LIST.find(({ domains }) =>
		domains.some((domain) => hostnameMatchesDomain(hostname, domain)),
	)
	return entry ? { label: entry.label, url } : null
}
