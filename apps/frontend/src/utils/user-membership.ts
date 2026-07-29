import { UserBadge } from '@modrinth/utils'

const PRIDE_26_MIDAS_DURATION_MS = 30 * 24 * 60 * 60 * 1000

type Pride26Campaign = {
	last_donated_at?: string | null
	has_badge?: boolean | null
	has_midas?: boolean | null
}

type UserWithMembership = {
	badges?: number | null
	campaigns?: {
		pride_26?: Pride26Campaign | null
	} | null
}

export function hasPride26Badge(user?: UserWithMembership | null) {
	return user?.campaigns?.pride_26?.has_badge === true
}

export function hasActivePride26Midas(user?: UserWithMembership | null, now = Date.now()) {
	const pride26Campaign = user?.campaigns?.pride_26

	if (!pride26Campaign?.has_midas || !pride26Campaign.last_donated_at) {
		return false
	}

	const lastDonatedAt = Date.parse(pride26Campaign.last_donated_at)

	if (!Number.isFinite(lastDonatedAt)) {
		return false
	}

	return lastDonatedAt + PRIDE_26_MIDAS_DURATION_MS > now
}

export function hasActiveMidas(user?: UserWithMembership | null, now = Date.now()) {
	return Boolean((user?.badges ?? 0) & UserBadge.MIDAS) || hasActivePride26Midas(user, now)
}
