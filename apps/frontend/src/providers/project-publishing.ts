import { normalizeProjectNagKind } from '@modrinth/moderation'
import { createContext, type ProjectPageContext } from '@modrinth/ui'
import { useLocalStorage } from '@vueuse/core'
import { computed, type Ref, watch } from 'vue'

import {
	canResubmitProjectForReview,
	canSubmitProjectForReview,
} from '~/composables/link-network-validation/submission'
import { isRejected } from '~/helpers/projects.js'
import { isThreadIssueVerified, threadIssueVerdict } from '~/helpers/thread-issues'

export const projectPublishingTones = {
	checking: 'neutral',
	unavailable: 'neutral',
	todo: 'warning',
	ready: 'success',
	'ready-to-resubmit': 'success',
	review: 'info',
	approved: 'success',
	unlisted: 'success',
	private: 'success',
	withheld: 'info',
	changes: 'warning',
} as const

export type ProjectPublishingState = keyof typeof projectPublishingTones

export function isProjectChecklistVisible(state: ProjectPublishingState, remaining: number) {
	return state !== 'checking' && state !== 'unavailable' && (state !== 'approved' || remaining > 0)
}

export const projectPublishingColors = {
	info: 'blue',
	success: 'green',
	warning: 'orange',
	neutral: undefined,
} as const

type PublishingSources = Pick<
	ProjectPageContext,
	| 'projectV2'
	| 'projectValidation'
	| 'projectValidationLoading'
	| 'projectLinksNetworkValidationLoading'
	| 'thread'
> & {
	canBypassValidation: Readonly<Ref<boolean>>
	submitting: Readonly<Ref<boolean>>
}

function createProjectPublishingContext({
	projectV2: project,
	projectValidation,
	projectValidationLoading,
	projectLinksNetworkValidationLoading,
	thread,
	canBypassValidation,
	submitting,
}: PublishingSources) {
	const rejected = computed(() => !!project.value && isRejected(project.value))
	const checking = computed(
		() => projectValidationLoading.value || projectLinksNetworkValidationLoading.value,
	)
	const nags = computed(() =>
		(projectValidation.value?.nags ?? []).filter((nag) => {
			const kind = normalizeProjectNagKind(nag.kind)
			return kind && kind !== 'moderator-feedback'
		}),
	)
	const issues = computed(() =>
		(thread.value?.issues ?? [])
			.filter((issue) => !isThreadIssueVerified(issue))
			.map((source) => ({ source, verdict: threadIssueVerdict(source) })),
	)
	const remaining = computed(
		() =>
			nags.value.filter((nag) => nag.severity === 'required').length +
			issues.value.filter((issue) => issue.verdict === 'open').length,
	)
	const validationReady = computed(() =>
		canSubmitProjectForReview(projectValidation.value, checking.value),
	)
	const canSubmit = computed(
		() =>
			!!project.value &&
			!submitting.value &&
			(canBypassValidation.value || validationReady.value) &&
			(!rejected.value || canResubmitProjectForReview(thread.value)),
	)
	const state = computed<ProjectPublishingState>(() => {
		const status = project.value?.status
		if (!status) return 'unavailable'
		if (rejected.value && (canSubmit.value || submitting.value)) return 'ready-to-resubmit'
		if (!projectValidation.value) return checking.value ? 'checking' : 'unavailable'
		switch (status) {
			case 'processing':
				return 'review'
			case 'archived':
				return 'approved'
			case 'approved':
			case 'unlisted':
			case 'private':
			case 'withheld':
				return status
			case 'rejected':
				return 'changes'
		}
		if (remaining.value) return 'todo'
		if (status === 'draft') return validationReady.value || submitting.value ? 'ready' : 'checking'
		return 'unavailable'
	})

	const dismissedBanners = useLocalStorage<Partial<Record<string, 'approved' | 'withheld'>>>(
		'project-publishing-dismissed-banners',
		{},
		{ initOnMounted: true, writeDefaults: false },
	)
	const projectId = computed(() => project.value?.id)
	const dismissedState = computed(() =>
		projectId.value ? dismissedBanners.value[projectId.value] : undefined,
	)
	const bannerDismissed = computed(() => dismissedState.value === state.value)

	watch(
		[projectId, () => project.value?.status, state, dismissedState],
		([id, status, currentState, dismissed]) => {
			if (!id || !status || !dismissed) return

			const validationMissing = currentState === 'checking' || currentState === 'unavailable'
			const statusMatches =
				status === dismissed || (status === 'archived' && dismissed === 'approved')
			if (dismissed !== currentState && !(validationMissing && statusMatches)) {
				dismissedBanners.value = Object.fromEntries(
					Object.entries(dismissedBanners.value).filter(([projectId]) => projectId !== id),
				)
			}
		},
		{ immediate: true },
	)

	function dismissBanner() {
		const id = projectId.value
		const currentState = state.value
		if (id && (currentState === 'approved' || currentState === 'withheld')) {
			dismissedBanners.value[id] = currentState
		}
	}

	return {
		bannerDismissed,
		canSubmit,
		dismissBanner,
		issues,
		nags,
		rejected,
		remaining,
		state,
		submitting,
	}
}

const [injectProjectPublishing, providePublishingContext] = createContext<
	ReturnType<typeof createProjectPublishingContext>
>('ProjectPage', 'projectPublishing')

export { injectProjectPublishing }

export function provideProjectPublishing(sources: PublishingSources) {
	return providePublishingContext(createProjectPublishingContext(sources))
}
