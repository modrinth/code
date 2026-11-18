import { normalizeProjectNagKind } from '@modrinth/moderation'
import { createContext, type ProjectPageContext } from '@modrinth/ui'
import { computed, type Ref } from 'vue'

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
	default: 'neutral',
} as const

export type ProjectPublishingState = keyof typeof projectPublishingTones

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
> & { submitting: Readonly<Ref<boolean>> }

function createProjectPublishingContext({
	projectV2: project,
	projectValidation,
	projectValidationLoading,
	projectLinksNetworkValidationLoading,
	thread,
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
	const canSubmit = computed(
		() =>
			!!project.value &&
			canSubmitProjectForReview(projectValidation.value, checking.value || submitting.value) &&
			(!rejected.value || canResubmitProjectForReview(thread.value)),
	)
	const state = computed<ProjectPublishingState>(() => {
		const status = project.value?.status
		if (!status) return 'default'
		if (rejected.value && (canSubmit.value || submitting.value)) return 'ready-to-resubmit'
		if (status === 'draft' && submitting.value) return 'ready'
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
		if (status === 'draft') return canSubmit.value ? 'ready' : 'checking'
		return 'default'
	})

	return { canSubmit, issues, nags, rejected, remaining, state, submitting }
}

const [injectProjectPublishing, providePublishingContext] = createContext<
	ReturnType<typeof createProjectPublishingContext>
>('ProjectPage', 'projectPublishing')

export { injectProjectPublishing }

export function provideProjectPublishing(sources: PublishingSources) {
	return providePublishingContext(createProjectPublishingContext(sources))
}
