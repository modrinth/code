import type { Labrinth } from '@modrinth/api-client'
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
import { sendThreadReply, type ThreadReply } from '~/helpers/thread-issues'
import { isStaff } from '~/helpers/users.js'

import { injectProjectReviewPageContext } from './index'
import { applyReviewDecision, type ReviewDecision } from './review-decision'
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
	const { project, threadQuery, disclosures } = injectProjectReviewPageContext()
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
				await applyReviewDecision(decision, client, () => assertCurrent(id), clearMessage)
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
		onError: (error) =>
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: error instanceof Error ? error.message : String(error),
				type: 'error',
			}),
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

	const canAddIssues = computed(() => canSubmit.value && !!selectedIssues())

	async function submitReview(status?: ProjectStatus) {
		const current = project.value
		const thread = threadQuery.data.value
		if (!canSubmit.value || !current || !thread || messages.generating.value) return
		if (!status && !canAddIssues.value) return
		let mutationStarted = false
		try {
			if (disclosures.hasChanges.value || disclosures.saving.value)
				throw new Error(formatMessage(errors.unsaved))
			if (panels.validationErrors.value.length) throw new Error(formatMessage(errors.missing))
			if (status && ['approved', 'unlisted', 'private'].includes(status) && !canApprove.value)
				throw new Error(formatMessage(errors.unresolved))
			const decision: ReviewDecision = {
				projectId: current.id,
				threadId: current.thread_id,
				status,
				body: status ? draft.value : '',
				images: status ? [...uploadedImages.value] : [],
				privateMessage: false,
				facetUpdates: status ? [...previousIssues.facetUpdates.value] : [],
				issues: selectedIssues(),
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
		loadingAction,
		submit,
		submitDecision: (status: ProjectStatus) => submitReview(status),
		submitIssues: () => submitReview(),
		uploadImage: (file: File) => upload.mutateAsync({ file, id: project.value?.id ?? '' }),
	}
}
