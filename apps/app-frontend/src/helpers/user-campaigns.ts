import type { Labrinth } from '@modrinth/api-client'

const MIDAS_BITFLAG = 1 << 0
const PRIDE_26_MIDAS_DURATION_MS = 30 * 24 * 60 * 60 * 1000

type Pride26Campaign = Labrinth.Users.v3.Pride26CampaignDonation | null | undefined

export function hasMidasBadge(user?: { badges?: number } | null) {
	return !!user?.badges && (user.badges & MIDAS_BITFLAG) === MIDAS_BITFLAG
}

export function hasPride26Badge(campaign: Pride26Campaign) {
	return campaign?.has_badge === true
}

export function hasActivePride26Midas(campaign: Pride26Campaign, now = Date.now()) {
	if (campaign?.has_midas !== true) {
		return false
	}

	const donatedAt = new Date(campaign.last_donated_at).getTime()
	return Number.isFinite(donatedAt) && donatedAt + PRIDE_26_MIDAS_DURATION_MS > now
}
