import type { Labrinth } from '@modrinth/api-client'

export type UpdateChannelPolicy = 'release' | 'all'

export function allowsUpdateChannel(
	version: Pick<Labrinth.Versions.v2.Version, 'version_type'>,
	policy: UpdateChannelPolicy,
) {
	return policy === 'all' || version.version_type === 'release'
}

export function newestEligibleUpdate(
	versions: Labrinth.Versions.v2.Version[],
	currentVersionId: string,
	currentPublishedAt: string | null | undefined,
	policy: UpdateChannelPolicy,
) {
	const currentTime = currentPublishedAt ? new Date(currentPublishedAt).getTime() : Number.NaN

	return (
		[...versions]
			.sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime())
			.find((version) => {
				if (version.id === currentVersionId) return false
				if (!allowsUpdateChannel(version, policy)) return false
				if (Number.isNaN(currentTime)) return true
				return new Date(version.date_published).getTime() > currentTime
			}) ?? null
	)
}
