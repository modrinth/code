export interface MessageDescriptor {
	id: string
	defaultMessage?: string
	description?: string
}

export interface LinkCheckContext {
	url: string | undefined
	field: string
	generalContent?: boolean

	[key: string]: unknown
}

export interface LinkCheckResult {
	severity: 'valid' | 'warn' | 'error'
	message?: MessageDescriptor
	values?: Record<string, unknown>
}

export interface BlockedProjectLink extends Record<string, unknown> {
	label: string
	url: string
}

export type FieldMatcher =
	| string
	| string[]
	| ((field: string, context: LinkCheckContext) => boolean)

export type LinkCheckVerify = (
	match: RegExpMatchArray,
	context: LinkCheckContext,
) => LinkCheckResult

export type RemoteLinkCheckVerify = (
	match: RegExpMatchArray,
	context: LinkCheckContext,
) => Promise<LinkCheckResult>

export type LinkCheckMatcher =
	| RegExp
	| ((remaining: string) => number | null | Promise<number | null>)

export interface LinkCheckNode {
	when: LinkCheckMatcher
	label?: string
	unrecognizedSeverity?: 'error' | 'warn'
	unrecognizedMessage?: MessageDescriptor
	forMatchers?: FieldMatcher[]
	verifyMatch?: LinkCheckVerify | RemoteLinkCheckVerify
	isRemoteVerification?: boolean
	childNodes?: LinkCheckNode[]
	isTransparent?: boolean
	isFallback?: boolean
}

export interface LinkCheckBuilder {
	when: LinkCheckMatcher
	label?: string

	for(fields: FieldMatcher): LinkCheckBuilder

	verify(fn: LinkCheckVerify): LinkCheckBuilder
	verifyRemotely(fn: RemoteLinkCheckVerify): LinkCheckBuilder

	children(...shapes: LinkCheckChildShape[]): LinkCheckBuilder

	severity(value: 'error' | 'warn'): LinkCheckBuilder

	message(descriptor: MessageDescriptor): LinkCheckBuilder

	transparent(): LinkCheckBuilder

	fallback(): LinkCheckBuilder

	warn(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckBuilder

	error(message: MessageDescriptor, values?: Record<string, unknown>): LinkCheckBuilder
}

export type LinkCheckChildShape =
	| LinkCheckNode
	| LinkCheckBuilder
	| RegExp
	| string
	| ((remaining: string) => number | null | Promise<number | null>)

export interface MatchResult {
	node: LinkCheckNode
	match: RegExpMatchArray
	expectedChild?: LinkCheckNode
}
