export { EXTERNAL_LINKS_BLOCK_LIST, URL_SHORTENERS } from './block-list.ts'
export { PROJECT_LINK_DOMAIN_LIST } from './domain-list.ts'
export {
	getBlockedProjectExternalLink,
	getLinkHostname,
	hostnameMatchesDomain,
	isCommonProjectLink,
	isDiscordLink,
	isInappropriateLicenseLink,
} from './syntax-checks.ts'
export type {
	BlockedProjectLink,
	LinkCheckContext,
	LinkCheckResult,
	MessageDescriptor,
} from './types.ts'
export { validateLink, validateLinkSyntax } from './validation.ts'
