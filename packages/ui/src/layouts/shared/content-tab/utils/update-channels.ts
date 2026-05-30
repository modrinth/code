import type { Labrinth } from '@modrinth/api-client'

export type UpdateChannelPolicy = 'release' | 'beta' | 'alpha'

const channelRank: Record<UpdateChannelPolicy, number> = {
	release: 0,
	beta: 1,
	alpha: 2,
}

function normalizeChannel(versionType: string): UpdateChannelPolicy {
	if (versionType === 'alpha' || versionType === 'beta') return versionType
	return 'release'
}

function effectiveUpdateChannel(
	policy: UpdateChannelPolicy,
	currentVersionType?: string | null,
): UpdateChannelPolicy {
	if (!currentVersionType) return policy

	const currentChannel = normalizeChannel(currentVersionType)
	return channelRank[currentChannel] > channelRank[policy] ? currentChannel : policy
}

function channelFallbacks(policy: UpdateChannelPolicy): UpdateChannelPolicy[][] {
	switch (policy) {
		case 'release':
			return [['release'], ['beta'], ['alpha']]
		case 'beta':
			return [['release', 'beta'], ['alpha']]
		case 'alpha':
			return [['release', 'beta', 'alpha']]
	}
}

export function allowsUpdateChannel(
	version: Pick<Labrinth.Versions.v2.Version, 'version_type'>,
	policy: UpdateChannelPolicy,
	currentVersionType?: string | null,
) {
	const effectivePolicy = effectiveUpdateChannel(policy, currentVersionType)
	return channelFallbacks(effectivePolicy)[0].includes(normalizeChannel(version.version_type))
}

export function newestEligibleUpdate(
	versions: Labrinth.Versions.v2.Version[],
	currentVersionId: string,
	currentPublishedAt: string | null | undefined,
	policy: UpdateChannelPolicy,
	currentVersionType?: string | null,
) {
	const currentTime = currentPublishedAt ? new Date(currentPublishedAt).getTime() : Number.NaN
	const sortedVersions = [...versions].sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	)
	const effectivePolicy = effectiveUpdateChannel(policy, currentVersionType)

	for (const versionTypes of channelFallbacks(effectivePolicy)) {
		if (!versions.some((version) => versionTypes.includes(normalizeChannel(version.version_type)))) {
			continue
		}

		return (
			sortedVersions.find((version) => {
				if (version.id === currentVersionId) return false
				if (!versionTypes.includes(normalizeChannel(version.version_type))) return false
				if (Number.isNaN(currentTime)) return true
				return new Date(version.date_published).getTime() > currentTime
			}) ?? null
		)
	}

	return null
}
