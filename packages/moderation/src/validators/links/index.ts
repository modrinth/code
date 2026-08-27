export { PROJECT_LINK_BLOCK_LIST } from './block-list.ts'
export { PROJECT_LINK_DOMAIN_LIST } from './domain-list.ts'
export {
	getBlockedProjectContentLink,
	getBlockedProjectExternalLink,
	getLinkHostname,
	hostnameMatchesDomain,
	isCommonProjectLink,
	isDiscordLink,
	isInappropriateLicenseLink,
	isLinkShortener,
} from './syntax-checks.ts'
export type {
	BlockedProjectLink,
	LinkCheckContext,
	LinkCheckResult,
	MessageDescriptor,
} from './types.ts'
export { validateLink, validateLinkSyntax } from './validation.ts'
