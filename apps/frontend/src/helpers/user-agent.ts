export const VISITOR_USER_AGENT_HEADER = 'X-Forwarded-User-Agent'

export function getFrontendUserAgent(commitHash: string): string {
	return `modrinth/frontend/${commitHash || 'unknown'} (support@modrinth.com)`
}
