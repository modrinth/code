import { defineMessage } from '@modrinth/ui'

import {
	fullAllocation,
	type MemberWeight,
	toAllocationUnits,
} from './revenue-split-editor/allocation'
import type { MemberDraft } from './types'

export const maxPayoutWeight = 5000

export function isValidPayoutWeight(weight: number) {
	return Number.isFinite(weight) && weight >= 0 && weight <= maxPayoutWeight
}

const revenueColors = [
	'var(--color-brand)',
	'var(--color-blue)',
	'var(--color-purple)',
	'var(--color-orange)',
	'var(--color-red)',
	'#2cbab5',
	'#ec87bd',
	'#d8b34a',
	'#8499ed',
	'#93b85c',
]

export const revenuePercentMessage = defineMessage({
	id: 'project.settings.members.weights.percent',
	defaultMessage: '{share, number, ::percent .##}',
})

export function revenueWeight(member: MemberDraft) {
	const weight = Number(member.payouts_split)
	return Number.isFinite(weight) ? Math.max(0, weight) : 0
}

export function revenueTotal(members: MemberDraft[]) {
	return members.reduce((total, member) => total + (member.accepted ? revenueWeight(member) : 0), 0)
}

export function payoutShare(weight: number, total: number, memberCount: number) {
	if (memberCount === 0) return 0
	return total > 0 ? weight / total : 1 / memberCount
}

export function revenueShare(member: MemberDraft, total: number, acceptedCount: number) {
	return member.accepted ? payoutShare(revenueWeight(member), total, acceptedCount) : 0
}

export function revenueMembersWithOverrides(
	members: MemberDraft[],
	overrides: ReadonlyMap<string, boolean>,
): MemberDraft[] {
	return members.map((member) => {
		const override = overrides.get(member.user.id)
		if (override === undefined || !member.organizationDefaults) return member
		return override
			? { ...member, override }
			: { ...member, ...member.organizationDefaults, override }
	})
}

function roundShares(shares: number[], total: number) {
	const allocations = shares.map((exact, index) => {
		const units = Math.floor(exact)
		return { index, units, remainder: exact - units }
	})
	const roundingUnits = total - allocations.reduce((sum, row) => sum + row.units, 0)
	const roundedUp = new Set(
		[...allocations]
			.sort((a, b) => b.remainder - a.remainder)
			.slice(0, roundingUnits)
			.map(({ index }) => index),
	)
	return allocations.map(({ index, units }) => (units + Number(roundedUp.has(index))) / 100)
}

export function revenueAllocation(
	members: MemberDraft[],
	{ editing = false } = {},
): MemberWeight[] {
	const total = revenueTotal(members)
	const available =
		editing && total === 0 && members.some((member) => member.override === false)
			? 0
			: fullAllocation
	const percentages = roundShares(
		members.map((member) => revenueShare(member, total, members.length) * available),
		available,
	)

	return members.map((member, index) => ({
		id: member.user.id,
		name: member.user.username,
		color: revenueColors[index % revenueColors.length],
		percentage: percentages[index] ?? 0,
		fixed: member.override === false,
		organizationMember: member.organizationDefaults !== undefined,
	}))
}

function revenueDraftRows(members: MemberDraft[], allocation: MemberWeight[]) {
	const initial = revenueAllocation(members, { editing: true })
	const rowsById = new Map(allocation.map((row) => [row.id, row]))
	const hasInheritedWeight = inheritedRevenueWeight(members) > 0
	return members.map((member, index) => {
		const userId = member.user.id
		const originalShare = toAllocationUnits(initial[index]?.percentage ?? 0)
		const row = rowsById.get(userId)
		const share = row ? toAllocationUnits(row.percentage) : originalShare
		const inherited = member.override === false
		return {
			userId,
			share,
			originalWeight: revenueWeight(member),
			inherited,
			edited:
				!inherited &&
				(share !== originalShare || (hasInheritedWeight && row?.shareEdited === true)),
		}
	})
}

export function inheritedRevenueWeight(members: MemberDraft[]) {
	return revenueTotal(members.filter((member) => member.override === false))
}

export function maxInheritedRevenueShare(
	members: MemberDraft[],
	allocation: MemberWeight[],
	userId: string,
) {
	const otherRows = revenueDraftRows(members, allocation).filter((row) => row.userId !== userId)
	const otherEdits = otherRows.filter((row) => row.edited)
	const unchangedWeight = otherRows.reduce(
		(sum, row) => sum + (row.edited ? 0 : row.originalWeight),
		0,
	)
	const available = fullAllocation - otherEdits.reduce((sum, row) => sum + row.share, 0)
	const largestOtherShare = otherEdits.reduce((largest, row) => Math.max(largest, row.share), 0)
	const limitForTarget = Math.floor(
		(available * maxPayoutWeight) / (unchangedWeight + maxPayoutWeight),
	)
	const minimumRemaining = Math.max(
		1,
		Math.ceil((unchangedWeight * largestOtherShare) / maxPayoutWeight),
	)

	return Math.max(0, Math.min(limitForTarget, available - minimumRemaining)) / 100
}

export function withInheritedRevenueShares(members: MemberDraft[], allocation: MemberWeight[]) {
	if (inheritedRevenueWeight(members) === 0) return allocation

	const rows = revenueDraftRows(members, allocation)
	const unchanged = rows.filter((row) => !row.edited)
	const unchangedWeight = unchanged.reduce((sum, row) => sum + row.originalWeight, 0)
	const editedShare = rows.reduce((sum, row) => sum + (row.edited ? row.share : 0), 0)
	const remaining = Math.max(0, fullAllocation - editedShare)
	const percentages = roundShares(
		unchanged.map((row) => (row.originalWeight / unchangedWeight) * remaining),
		remaining,
	)
	const sharesById = new Map(
		unchanged.map(({ userId }, index) => [userId, percentages[index] ?? 0]),
	)
	return allocation.map((row) => {
		const percentage = sharesById.get(row.id)
		return percentage !== undefined && percentage !== row.percentage ? { ...row, percentage } : row
	})
}

export function revenueWeightUpdates(members: MemberDraft[], allocation: MemberWeight[]) {
	const rows = revenueDraftRows(members, allocation)
	const edited = rows.filter((row) => row.edited)
	const hasInheritedWeight = rows.some((row) => row.inherited && row.originalWeight > 0)
	const unchangedWeight = rows.reduce((sum, row) => sum + (row.edited ? 0 : row.originalWeight), 0)
	const editedShare = edited.reduce((sum, row) => sum + row.share, 0)
	const remainingShare = fullAllocation - editedShare
	const preserveRemainingShare = unchangedWeight > 0 && (remainingShare > 0 || hasInheritedWeight)
	const largestShare = edited.reduce((largest, row) => Math.max(largest, row.share), 0)
	if (
		!hasInheritedWeight &&
		preserveRemainingShare &&
		(unchangedWeight * largestShare) / remainingShare > maxPayoutWeight
	) {
		const editable = rows.filter((row) => !row.inherited)
		const largestEditableShare = editable.reduce((largest, row) => Math.max(largest, row.share), 0)
		return editable.flatMap(({ userId, share, originalWeight }) => {
			const weight = (share / largestEditableShare) * maxPayoutWeight
			return originalWeight === weight ? [] : [{ userId, weight }]
		})
	}

	const weightBudget = edited.reduce((total, row) => total + row.originalWeight, 0) || 100
	const scaleToLimit =
		editedShare > 0 && (largestShare / editedShare) * weightBudget > maxPayoutWeight

	return edited.flatMap(({ userId, share, originalWeight }) => {
		let weight = 0
		if (share > 0 && preserveRemainingShare) {
			weight = remainingShare > 0 ? (unchangedWeight * share) / remainingShare : Infinity
		} else if (share > 0 && scaleToLimit) {
			weight = (share / largestShare) * maxPayoutWeight
		} else if (share > 0) {
			weight = (share / editedShare) * weightBudget
		}
		return originalWeight === weight ? [] : [{ userId, weight }]
	})
}
