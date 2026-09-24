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

type ProjectStatus = Labrinth.Projects.v2.ProjectStatus
type ReviewEditorMode = 'reply' | 'note'

export const [injectReviewSubmission, provideReviewSubmission] =
	createContext<ReturnType<typeof createReviewSubmission>>('ReviewSubmission')

export function createReviewSubmission(
	messages: ReturnType<typeof createReviewMessages>,
	panels: ReturnType<typeof createReviewPanels>,
) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { formatMessage } = useVIntl()
	const { addNotification } = injectNotificationManager()
	const auth = useAuthState()
	const { project, threadQuery, disclosures } = injectProjectReviewPageContext()
	const draft = ref('')
	let pendingDecision: { id: string; status: ProjectStatus; body: string } | undefined
	const uploadedImages = ref<string[]>([])
	let disposed = false
	watch(
		() => project.value?.id,
		() => {
			draft.value = ''
			uploadedImages.value = []
			pendingDecision = undefined
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
		conflict: {
			id: 'project-review.corrections.conflicts',
			defaultMessage: 'Resolve conflicting corrections before submitting a review decision.',
		},
		changed: {
			id: 'project-review.corrections.project-changed',
			defaultMessage: 'The selected project changed. Review the corrections again.',
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

	const submission = useMutation({
		mutationFn: async ({
			id,
			threadId,
			body,
			images,
			privateMessage,
			status,
			corrections,
		}: {
			id: string
			threadId: string
			body: string
			images: string[]
			privateMessage: boolean
			status?: ProjectStatus
			corrections: typeof panels.corrections.value | undefined
		}) => {
			assertCurrent(id)
			if (status) {
				if (disclosures.hasChanges.value || disclosures.saving.value)
					throw new Error(formatMessage(errors.unsaved))
				if (panels.validationErrors.value.length) throw new Error(formatMessage(errors.missing))
			}
			if (corrections) {
				if (corrections.conflicts.length) throw new Error(formatMessage(errors.conflict))
				if (
					Object.keys(corrections.versions).some(
						(version) => !project.value?.versions.includes(version),
					)
				)
					throw new Error(formatMessage(errors.changed))
				if (Object.keys(corrections.project).length)
					await client.labrinth.projects_v3.edit(id, corrections.project)
				for (const [versionId, patch] of Object.entries(corrections.versions)) {
					assertCurrent(id)
					if (Object.keys(patch).length)
						await client.labrinth.versions_v3.modifyVersion(versionId, patch)
				}
			}
			assertCurrent(id)
			if (body) {
				await client.labrinth.threads_v3.sendMessage(threadId, {
					body: {
						type: 'text',
						body,
						private: privateMessage,
						associated_images: images,
					},
				})
				if (status) {
					pendingDecision = { id, status, body }
				} else if (!disposed && project.value?.id === id) {
					draft.value = ''
					uploadedImages.value = []
				}
			}
			if (status) {
				assertCurrent(id)
				await client.labrinth.projects_v3.edit(id, { status })
				pendingDecision = undefined
			}
		},
		onError: (error) =>
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: error instanceof Error ? error.message : String(error),
				type: 'error',
			}),
		onSettled: async (_, __, { id, threadId, corrections }) => {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ['project', 'v3', id] }),
				queryClient.invalidateQueries({ queryKey: ['project', 'v2', id] }),
				queryClient.invalidateQueries({ queryKey: ['project', id] }),
				queryClient.invalidateQueries({ queryKey: ['thread', threadId] }),
				...Object.keys(corrections?.versions ?? {}).map((versionId) =>
					queryClient.invalidateQueries({ queryKey: ['version', versionId] }),
				),
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
				corrections: undefined,
			})
			.catch(() => undefined)
	}

	async function submitDecision(status: ProjectStatus) {
		const current = project.value
		if (!canSubmit.value || !current || messages.generating.value) return
		const body = messages.generated.value
		const retryStatus =
			pendingDecision?.id === current.id &&
			pendingDecision.status === status &&
			pendingDecision.body === body
		await submission
			.mutateAsync({
				id: current.id,
				threadId: current.thread_id,
				body: retryStatus || !body.trim() ? '' : body,
				images: [],
				privateMessage: false,
				status,
				corrections:
					!retryStatus && panels.correctionsRequested.value ? panels.corrections.value : undefined,
			})
			.catch(() => undefined)
	}

	return {
		draft,
		pending,
		canSubmit,
		loadingAction,
		submit,
		submitDecision,
		uploadImage: (file: File) => upload.mutateAsync({ file, id: project.value?.id ?? '' }),
	}
}
