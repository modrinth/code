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
import { isStaff } from '~/helpers/users.js'

import { injectProjectReviewPageContext } from './index'
import type { createReviewMessages } from './review-messages'
import type { createReviewPanels } from './review-panels'
import type { createReviewPreviousIssues } from './review-previous-issues'
import type { createReviewSession } from './review-session'

type ProjectStatus = Labrinth.Projects.v2.ProjectStatus
type ReviewEditorMode = 'reply' | 'note'
type NewThreadIssues = Labrinth.Threads.v3.NewThreadIssues
type IssueUpdate = { id: string; data: Labrinth.Threads.v3.EditThreadIssue }

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
	const pendingDecision = ref<{
		id: string
		status: ProjectStatus
		body: string
		messageSent: boolean
		issues?: NewThreadIssues
		issueUpdates: IssueUpdate[]
	}>()
	const uploadedImages = ref<string[]>([])
	let disposed = false
	watch(
		() => project.value?.id,
		() => {
			draft.value = ''
			uploadedImages.value = []
			pendingDecision.value = undefined
		},
		{ flush: 'sync' },
	)
	onScopeDispose(() => {
		disposed = true
	})

	const errors = defineMessages({
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
		for (const { id, facets } of panels.activeIssues.value) {
			if (!facets || previousIds.has(id)) continue
			selected.push({
				why: {
					issue_id: id,
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
			threadId,
			body,
			images,
			privateMessage,
			status,
			statusAlreadyApplied = false,
			issues,
			issueUpdates = [],
		}: {
			id: string
			threadId: string
			body: string
			images: string[]
			privateMessage: boolean
			status?: ProjectStatus
			statusAlreadyApplied?: boolean
			issues?: NewThreadIssues
			issueUpdates?: IssueUpdate[]
		}) => {
			assertCurrent(id)
			if (status && !statusAlreadyApplied) {
				if (disclosures.hasChanges.value || disclosures.saving.value)
					throw new Error(formatMessage(errors.unsaved))
				if (panels.validationErrors.value.length) throw new Error(formatMessage(errors.missing))
			}
			assertCurrent(id)
			if (status && !statusAlreadyApplied) {
				await client.labrinth.projects_v3.edit(id, { status })
				pendingDecision.value = { id, status, body, messageSent: false, issues, issueUpdates }
			}
			assertCurrent(id)
			if (body && (!status || !pendingDecision.value?.messageSent)) {
				await client.labrinth.threads_v3.sendMessage(threadId, {
					body: {
						type: 'text',
						body,
						private: privateMessage,
						associated_images: images,
					},
				})
				if (status && pendingDecision.value) pendingDecision.value.messageSent = true
				else if (!status && !disposed && project.value?.id === id) {
					draft.value = ''
					uploadedImages.value = []
				}
			}
			assertCurrent(id)
			if (status) {
				for (const issue of issueUpdates) {
					assertCurrent(id)
					await client.labrinth.threads_v3.editIssue(issue.id, issue.data)
					assertCurrent(id)
					if (pendingDecision.value) {
						pendingDecision.value.issueUpdates = pendingDecision.value.issueUpdates.filter(
							({ id }) => id !== issue.id,
						)
					}
				}
			}
			assertCurrent(id)
			if (status && issues) await client.labrinth.threads_v3.createIssues(threadId, issues)
			if (status) pendingDecision.value = undefined
			if (status) session.clearProject(id)
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
	})
	const pending = computed(() => submission.isPending.value || upload.isPending.value)
	const canSubmit = computed(
		() =>
			isStaff(auth.value.user) &&
			!!project.value &&
			threadQuery.data.value?.id === project.value.thread_id &&
			!pending.value,
	)
	const loadingAction = computed(() =>
		submission.isPending.value
			? (submission.variables.value?.status ??
				(submission.variables.value?.privateMessage ? 'note' : 'reply'))
			: undefined,
	)
	const pendingDecisionStatus = computed(() => {
		const currentPendingDecision = pendingDecision.value
		return currentPendingDecision?.id === project.value?.id
			? currentPendingDecision?.status
			: undefined
	})

	async function submit(mode: ReviewEditorMode = 'reply') {
		const current = project.value
		if (!canSubmit.value || !current || !draft.value.trim()) return
		await submission
			.mutateAsync({
				id: current.id,
				threadId: current.thread_id,
				body: draft.value,
				images: [...uploadedImages.value],
				privateMessage: mode === 'note',
			})
			.catch(() => undefined)
	}

	async function submitDecision(status: ProjectStatus) {
		const current = project.value
		if (!canSubmit.value || !current || messages.generating.value) return
		const generatedBody = messages.generated.value
		const normalizedBody = generatedBody.trim() ? generatedBody : ''
		const currentPendingDecision = pendingDecision.value
		const statusAlreadyApplied =
			currentPendingDecision?.id === current.id && currentPendingDecision.status === status
		const body = statusAlreadyApplied ? currentPendingDecision.body : normalizedBody
		try {
			await submission.mutateAsync({
				id: current.id,
				threadId: current.thread_id,
				body,
				images: [],
				privateMessage: false,
				status,
				statusAlreadyApplied,
				issues: statusAlreadyApplied ? currentPendingDecision.issues : selectedIssues(),
				issueUpdates: statusAlreadyApplied
					? currentPendingDecision.issueUpdates
					: previousIssues.issueUpdates.value,
			})
			return true
		} catch {
			return false
		}
	}

	return {
		draft,
		pending,
		canSubmit,
		loadingAction,
		pendingDecisionStatus,
		submit,
		submitDecision,
		uploadImage: (file: File) => upload.mutateAsync({ file, id: project.value?.id ?? '' }),
	}
}
