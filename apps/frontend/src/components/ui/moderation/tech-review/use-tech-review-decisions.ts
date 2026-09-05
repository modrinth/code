import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { inject, type InjectionKey, type MaybeRefOrGetter, reactive, toValue } from 'vue'

import { canUpdateGlobalDetail, decisionToVerdict, statusMatchesDecision } from './helpers'
import type { DetailDecision, DetailDecisionScope, FlagItem, FlattenedFileReport } from './types'

export function useTechReviewDecisions(reports: MaybeRefOrGetter<FlattenedFileReport[]>) {
	const client = injectModrinthClient()

	const detailDecisions = reactive<Map<string, DetailDecision>>(new Map())
	const detailDecisionScopes = reactive<Map<string, DetailDecisionScope>>(new Map())
	const updatingDetails = reactive<Set<string>>(new Set())
	const updatingGlobalDetailKeys = reactive<Set<string>>(new Set())

	function getDetailDecision(
		detailId: string,
		backendStatus: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
	): DetailDecision {
		const localDecision = detailDecisions.get(detailId)
		if (localDecision) return localDecision
		if (backendStatus === 'safe') return 'safe'
		if (backendStatus === 'unsafe') return 'malware'
		return 'pending'
	}

	function isPreReviewed(
		detailId: string,
		backendStatus: Labrinth.TechReview.Internal.DelphiReportIssueStatus,
	): boolean {
		return (
			(backendStatus === 'safe' || backendStatus === 'unsafe') && !detailDecisions.has(detailId)
		)
	}

	function getFileMarkedCount(file: FlattenedFileReport): number {
		let count = 0
		for (const issue of file.issues) {
			for (const detail of issue.details) {
				if (getDetailDecision(detail.id, detail.status) !== 'pending') count++
			}
		}
		return count
	}

	function getMarkedFlagsCount(flags: FlagItem[]): number {
		return flags.filter((f) => getDetailDecision(f.detail.id, f.detail.status) !== 'pending').length
	}

	function isDetailGloballyPassed(detail: Labrinth.TechReview.Internal.ReportIssueDetail): boolean {
		if (detailDecisionScopes.get(detail.id) === 'global') {
			return detailDecisions.get(detail.id) === 'safe'
		}

		return detail.global_status === 'safe'
	}

	function isDetailGloballyResolved(
		detail: Labrinth.TechReview.Internal.ReportIssueDetail,
	): boolean {
		if (detailDecisionScopes.get(detail.id) === 'global') {
			return detailDecisions.get(detail.id) !== 'pending'
		}

		return detail.global_status === 'safe' || detail.global_status === 'unsafe'
	}

	function applyDecisionToRelatedDetails(
		detailIds: string[],
		decision: DetailDecision,
		scope: DetailDecisionScope,
	): { otherMatchedCount: number } {
		const allDetails = toValue(reports).flatMap((report) =>
			report.issues.flatMap((issue) => issue.details),
		)
		const selectedDetailIds = new Set(detailIds)
		const updatedDetailIds = new Set<string>()

		for (const detailId of detailIds) {
			const detail = allDetails.find((candidate) => candidate.id === detailId)
			const matchingDetails = detail?.key
				? allDetails.filter((candidate) => candidate.key === detail.key)
				: detail
					? [detail]
					: []

			if (matchingDetails.length === 0) {
				detailDecisions.set(detailId, decision)
				detailDecisionScopes.set(detailId, scope)
				updatedDetailIds.add(detailId)
				continue
			}

			for (const matchingDetail of matchingDetails) {
				detailDecisions.set(matchingDetail.id, decision)
				detailDecisionScopes.set(matchingDetail.id, scope)
				updatedDetailIds.add(matchingDetail.id)
			}
		}

		return {
			otherMatchedCount: [...updatedDetailIds].filter((id) => !selectedDetailIds.has(id)).length,
		}
	}

	function isDetailActionSelected(
		detail: Labrinth.TechReview.Internal.ReportIssueDetail,
		decision: DetailDecision,
		scope: DetailDecisionScope,
	): boolean {
		const localDecision = detailDecisions.get(detail.id)
		const localScope = detailDecisionScopes.get(detail.id)
		if (localDecision && localScope) {
			if (localDecision === 'pending') {
				if (localScope === 'local') {
					if (scope === 'local') return false
					return statusMatchesDecision(detail.global_status, decision)
				}

				if (scope === 'global') return false
				return statusMatchesDecision(detail.local_status, decision)
			}

			return localDecision === decision && localScope === scope
		}

		if (scope === 'global') {
			return statusMatchesDecision(detail.global_status, decision)
		}

		if (detail.global_status) return false

		return statusMatchesDecision(detail.local_status, decision)
	}

	function getToggledDetailVerdict(
		detail: Labrinth.TechReview.Internal.ReportIssueDetail,
		decision: Exclude<DetailDecision, 'pending'>,
		scope: DetailDecisionScope,
	): Labrinth.TechReview.Internal.DelphiReportIssueStatus {
		return isDetailActionSelected(detail, decision, scope) ? 'pending' : decisionToVerdict(decision)
	}

	function getDetailActionTooltip(
		detail: Labrinth.TechReview.Internal.ReportIssueDetail,
		decision: Exclude<DetailDecision, 'pending'>,
		scope: DetailDecisionScope,
	): string {
		const action = decision === 'safe' ? 'pass' : 'fail'
		const scopeLabel = scope === 'global' ? 'Global' : 'Local'

		if (scope === 'global' && !canUpdateGlobalDetail(detail)) {
			return 'Global verdict unavailable for generated trace keys'
		}

		if (isDetailActionSelected(detail, decision, scope)) {
			return `Unset ${scopeLabel.toLowerCase()} ${action}`
		}

		return `${scopeLabel} ${action}`
	}

	async function updateIssueDetails(
		data: {
			detail_id: string
			verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus
		}[],
	) {
		await client.request('/moderation/tech-review/issue-detail', {
			api: 'labrinth',
			version: 'internal',
			method: 'PATCH',
			body: data,
		})
	}

	async function updateGlobalIssueDetails(
		data: {
			detail_key: string
			verdict: Labrinth.TechReview.Internal.DelphiReportIssueStatus
		}[],
	) {
		await client.labrinth.tech_review_internal.updateGlobalIssueDetails(data)
	}

	return {
		updatingDetails,
		updatingGlobalDetailKeys,
		getDetailDecision,
		isPreReviewed,
		getFileMarkedCount,
		getMarkedFlagsCount,
		isDetailGloballyPassed,
		isDetailGloballyResolved,
		applyDecisionToRelatedDetails,
		isDetailActionSelected,
		getToggledDetailVerdict,
		getDetailActionTooltip,
		updateIssueDetails,
		updateGlobalIssueDetails,
	}
}

export type TechReviewDecisions = ReturnType<typeof useTechReviewDecisions>

export const TECH_REVIEW_DECISIONS_KEY: InjectionKey<TechReviewDecisions> =
	Symbol('techReviewDecisions')

export function injectTechReviewDecisions(): TechReviewDecisions {
	const decisions = inject(TECH_REVIEW_DECISIONS_KEY)
	if (!decisions) {
		throw new Error('Tech review decisions must be provided by ModerationTechRevCard')
	}
	return decisions
}
