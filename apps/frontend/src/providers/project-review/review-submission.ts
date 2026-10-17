import type { Labrinth } from '@modrinth/api-client'
import { moderationSettings } from '@modrinth/moderation'
import { IssuePriority } from '@modrinth/moderation/src/data/issues'
import {
	commonMessages,
	createContext,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, onScopeDispose, ref, watch } from 'vue'

import { useAuthState } from '~/composables/auth'
import { useModerationSettings } from '~/composables/moderation'
import { sendThreadReply, type ThreadReply } from '~/helpers/thread-issues'
import { isStaff } from '~/helpers/users.js'

import { injectProjectReviewPageContext } from './index'
import { applyReviewDecision, type ReviewDecision, ReviewIssuesError } from './review-decision'
import { ReviewLockError } from './review-lock'
import type { createReviewMessages } from './review-messages'
import type { createReviewPanels } from './review-panels'
import type { createReviewPreviousIssues } from './review-previous-issues'
import type { createReviewSession } from './review-session'

type ProjectStatus = Labrinth.Projects.v2.ProjectStatus
type ReviewEditorMode = 'reply' | 'note'
type NewThreadIssues = Labrinth.Threads.v3.NewThreadIssues

export const [injectReviewSubmission, provideReviewSubmission] =
	createContext<ReturnType<typeof createReviewSubmission>>('ReviewSubmission')

export function createReviewSubmission(
	messages: ReturnType<typeof createReviewMessages>,
	panels: ReturnType<typeof createReviewPanels>,
	session: ReturnType<typeof createReviewSession>,
	previousIssues: ReturnType<typeof createReviewPreviousIssues>,
) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { formatMessage } = useVIntl()
	const { addNotification } = injectNotificationManager()
	const auth = useAuthState()
	const settings = useModerationSettings()
	const sendIssuesAsThreadMessage = computed(() =>
		settings.value.get(moderationSettings.General.SendIssuesAsThreadMessage),
	)
	const { project, threadQuery, disclosures, lock } = injectProjectReviewPageContext()
	const draft = ref('')
	const uploadedImages = ref<string[]>([])
	let disposed = false
	watch(
		() => project.value?.id,
		() => {
			draft.value = ''
			uploadedImages.value = []
		},
		{ flush: 'sync' },
	)
	onScopeDispose(() => {
		disposed = true
	})

	const errors = defineMessages({
		sendIssuesFailed: {
			id: 'project-review.decision.send-issues-failed',
			defaultMessage: 'Failed to send issues thread message',
		},
		createIssuesFailed: {
			id: 'project-review.decision.create-issues-failed',
			defaultMessage: 'Failed to create issues',
		},
		issuesMessageTooLong: {
			id: 'project-review.decision.issues-message-too-long',
			defaultMessage: 'The issues thread message must be no larger than 65,536 bytes.',
		},
		submitFailed: {
			id: 'project-review.decision.submit-failed',
			defaultMessage: 'Failed to submit review',
		},
		unresolved: {
			id: 'project-review.decision.unresolved',
			defaultMessage:
				'Verify all remaining fixes and remove new findings before approving this project.',
		},
		unsaved: {
			id: 'project-review.disclosures.unsaved',
			defaultMessage: 'Save or reset your disclosure changes before changing the project status.',
		},
		missing: {
			id: 'project-review.corrections.missing-fields',
			defaultMessage: 'Complete the required review fields before submitting a review decision.',
		},
		changed: {
			id: 'project-review.corrections.project-changed',
			defaultMessage: 'The selected project changed. Review it again.',
		},
		imageType: {
			id: 'project-review.reply.image-type',
			defaultMessage: 'Choose a PNG, JPEG, GIF, or WebP image.',
		},
		imageSize: {
			id: 'project-review.reply.image-size',
			defaultMessage: 'Images must be smaller than 1 MiB.',
		},
	})

	function assertCurrent(id: string) {
		if (disposed || project.value?.id !== id) throw new Error(formatMessage(errors.changed))
	}

	function selectedIssues(): NewThreadIssues | undefined {
		const selected: Labrinth.Threads.v3.NewThreadIssue[] = [...previousIssues.recreatedIssues.value]
		const previousIds = previousIssues.associatedIssueIds.value
		const titles = new Map(panels.availableIssues.value.map(({ id, title }) => [id, title]))
		for (const { id, facets, locations } of panels.activeIssues.value) {
			if (previousIds.has(id)) continue
			const custom = panels.customIssues.value.find((issue) => issue.id === id)?.custom
			selected.push({
				why: {
					issue_id: custom?.id.trim() ?? id,
					...(custom ? { custom: { priority: custom.priority } } : {}),
					locations,
					title: titles.get(id) ?? id,
					message: messages.issueMessage(id),
					selection: panels.issueSelection(id),
				},
				facets,
			})
		}
		const [first, ...rest] = selected
		return first ? { issues: [first, ...rest] } : undefined
	}

	const issuesMessage = computed(() => {
		const previousIds = previousIssues.associatedIssueIds.value
		return [
			...previousIssues.appliedIssues.value.map((issue) => ({
				priority: previousIssues.cardIssue(issue).priority ?? IssuePriority.Default,
				message: previousIssues.reviewMessage(issue),
			})),
			...panels.activeIssues.value
				.filter(({ id }) => !previousIds.has(id))
				.map(({ id, issue }) => ({
					priority: issue.priority ?? IssuePriority.Default,
					message: messages.issueMessage(id),
				})),
		]
			.sort((a, b) => a.priority - b.priority)
			.map(({ message }) => message.trim())
			.filter(Boolean)
			.join('\n\n')
	})

	const submission = useMutation({
		mutationFn: async ({
			id,
			decision,
			reply,
			clearIssues,
		}: {
			id: string
			threadId: string
			decision?: ReviewDecision
			reply?: ThreadReply
			clearIssues?: boolean
		}) => {
			assertCurrent(id)
			const clearMessage = () => {
				assertCurrent(id)
				draft.value = ''
				uploadedImages.value = []
			}
			if (decision) {
				await lock.ensureEligible(id)
				const assertReview = () => {
					assertCurrent(id)
					lock.assertEligible(id)
				}
				await applyReviewDecision(decision, client, assertReview, clearMessage)
				session.clearProject(id)
			} else if (reply) {
				const issueIds = clearIssues ? panels.activeIssues.value.map(({ id }) => id) : []
				const appliedIssues = clearIssues ? [...previousIssues.appliedIssues.value] : []
				await sendThreadReply(reply, client, () => assertCurrent(id))
				clearMessage()
				for (const issue of appliedIssues) previousIssues.markNoLongerApplicable(issue)
				for (const issueId of issueIds) panels.removeIssue(issueId)
			}
		},
		onError: (error) => {
			addNotification({
				title: formatMessage(
					error instanceof ReviewIssuesError
						? error.mode === 'message'
							? errors.sendIssuesFailed
							: errors.createIssuesFailed
						: error instanceof ReviewLockError
							? errors.submitFailed
							: commonMessages.errorNotificationTitle,
				),
				text: error instanceof Error ? error.message : String(error),
				type: 'error',
			})
		},
		onSettled: async (_, __, { id, threadId }) => {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ['project', 'v3', id] }),
				queryClient.invalidateQueries({ queryKey: ['project', 'v2', id] }),
				queryClient.invalidateQueries({ queryKey: ['project', id] }),
				queryClient.invalidateQueries({ queryKey: ['thread', threadId] }),
			])
		},
	})

	const upload = useMutation({
		mutationFn: async ({ file, id }: { file: File; id: string }) => {
			const ext = file.type.split('/')[1]
			if (!['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(file.type))
				throw new Error(formatMessage(errors.imageType))
			if (file.size > 1024 * 1024) throw new Error(formatMessage(errors.imageSize))
			const response = await client.upload<{ id: string; url: string }>('/image', {
				api: 'labrinth',
				version: 3,
				file,
				params: { context: 'thread_message', ext },
			}).promise
			assertCurrent(id)
			uploadedImages.value = [...uploadedImages.value, response.id].slice(-10)
			return response.url
		},
		onError: (error) =>
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: error instanceof Error ? error.message : String(error),
				type: 'error',
			}),
	})
	const pending = computed(() => submission.isPending.value || upload.isPending.value)
	const canSubmit = computed(
		() =>
			isStaff(auth.value.user) &&
			!!project.value &&
			threadQuery.data.value?.id === project.value.thread_id &&
			!submission.isPending.value &&
			!upload.isPending.value,
	)
	const loadingAction = computed(() =>
		submission.isPending.value
			? submission.variables.value?.decision
				? (submission.variables.value.decision.status ?? 'issues')
				: submission.variables.value?.reply?.privateMessage
					? 'note'
					: submission.variables.value?.clearIssues
						? 'reply-clear'
						: 'reply'
			: undefined,
	)
	async function submit(mode: ReviewEditorMode = 'reply', clearIssues = false) {
		const current = project.value
		if (!canSubmit.value || !current || !draft.value.trim()) return
		await submission
			.mutateAsync({
				id: current.id,
				threadId: current.thread_id,
				clearIssues: mode === 'reply' && clearIssues,
				reply: {
					threadId: current.thread_id,
					body: draft.value,
					images: [...uploadedImages.value],
					privateMessage: mode === 'note',
				},
			})
			.catch(() => undefined)
	}

	const canApprove = computed(
		() => !previousIssues.hasUnresolvedFacets.value && panels.activeIssues.value.length === 0,
	)

	const canAddIssues = computed(
		() =>
			canSubmit.value &&
			!!(sendIssuesAsThreadMessage.value ? issuesMessage.value : selectedIssues()),
	)

	async function submitReview(status?: ProjectStatus) {
		const current = project.value
		const thread = threadQuery.data.value
		if (!canSubmit.value || !current || !thread || messages.generating.value) return
		if (!status && !canAddIssues.value) return
		let mutationStarted = false
		try {
			if (disclosures.hasChanges.value || disclosures.saving.value)
				throw new Error(formatMessage(errors.unsaved))
			if (
				panels.validationErrors.value.some(
					({ issueId, key }) =>
						issueId === 'categories-inaccurate' && key === 'categories-remove-tags',
				)
			)
				throw new Error('Please select tags to remove for inaccurate tags issue.')
			if (panels.validationErrors.value.length) throw new Error(formatMessage(errors.missing))
			if (status && ['approved', 'unlisted', 'private'].includes(status) && !canApprove.value)
				throw new Error(formatMessage(errors.unresolved))
			const messageMode = sendIssuesAsThreadMessage.value
			const message = messageMode ? issuesMessage.value : undefined
			if (message && new TextEncoder().encode(message).length > 65536)
				throw new Error(formatMessage(errors.issuesMessageTooLong))
			const facetUpdates = status
				? previousIssues.facetUpdates.value
				: previousIssues.replacementFacetUpdates.value
			const applicableFacetIds = new Set(
				previousIssues.appliedIssues.value.flatMap((issue) =>
					issue.facets
						.filter((facet) => previousIssues.isFacetApplicable(issue, facet))
						.map(({ id }) => id),
				),
			)
			const decision: ReviewDecision = {
				projectId: current.id,
				threadId: current.thread_id,
				status,
				body: status ? draft.value : '',
				images: status ? [...uploadedImages.value] : [],
				privateMessage: false,
				facetUpdates: messageMode
					? status
						? facetUpdates.filter(
								({ id, data }) => !data.moderator_verified || !applicableFacetIds.has(id),
							)
						: []
					: [...facetUpdates],
				issues: messageMode ? undefined : selectedIssues(),
				issuesMessage: message,
			}
			mutationStarted = true
			await submission.mutateAsync({
				id: current.id,
				threadId: current.thread_id,
				decision,
			})
			return true
		} catch (error) {
			if (!mutationStarted)
				addNotification({
					title: formatMessage(commonMessages.errorNotificationTitle),
					text: error instanceof Error ? error.message : String(error),
					type: 'error',
				})
			return false
		}
	}

	return {
		draft,
		pending,
		canSubmit,
		canApprove,
		canAddIssues,
		sendIssuesAsThreadMessage,
		loadingAction,
		submit,
		submitDecision: (status: ProjectStatus) => submitReview(status),
		submitIssues: () => submitReview(),
		uploadImage: (file: File) => upload.mutateAsync({ file, id: project.value?.id ?? '' }),
	}
}
