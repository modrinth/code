import type { Labrinth } from '@modrinth/api-client'
import {
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useDocumentVisibility } from '@vueuse/core'
import { computed, onMounted, onScopeDispose, type Ref, ref, shallowRef, useId, watch } from 'vue'

import { useAuthState } from '~/composables/auth'

type LockStatus = Labrinth.Moderation.Internal.LockStatusResponse
type LockSession = {
	projectId: string
	userId: string
	token: string
	queryKey: readonly ['moderation', 'lock', string, string, number]
	everOwned: boolean
	notifiedOwner?: string
	takeoverNotificationId?: string | number
	errorNotified: boolean
}
type LockOperation = {
	session: LockSession
	kind: 'acquire' | 'renew' | 'override' | 'release' | 'check'
	forSubmission?: boolean
}

export class ReviewLockError extends Error {}

export function useReviewLock(projectId: Ref<string>) {
	const api = injectModrinthClient().labrinth.moderation_internal
	const queryClient = useQueryClient()
	const auth = useAuthState()
	const config = useRuntimeConfig()
	const { formatMessage } = useVIntl()
	const { addNotification, removeNotification } = injectNotificationManager()
	const instanceId = useId()
	const mounted = ref(false)
	const visibility = useDocumentVisibility()
	const session = shallowRef<LockSession>()
	let generation = 0
	let renewalTimer: ReturnType<typeof setInterval> | undefined
	const messages = defineMessages({
		takenOver: {
			id: 'project-review.lock.taken-over',
			defaultMessage: '{username} has taken over moderating this project.',
		},
		unverified: {
			id: 'project-review.lock.unverified',
			defaultMessage: 'Unable to verify the moderation lock on the project.',
		},
		acquireFailed: {
			id: 'project-review.lock.acquire-failed',
			defaultMessage: 'Unable to acquire the moderation lock for this project.',
		},
		renewFailed: {
			id: 'project-review.lock.renew-failed',
			defaultMessage: 'Unable to renew the moderation lock for this project.',
		},
		takeOverFailed: {
			id: 'project-review.lock.take-over-failed',
			defaultMessage: 'Unable to take over moderating this project.',
		},
		unknownModerator: {
			id: 'project-review.lock.unknown-moderator',
			defaultMessage: 'Another moderator',
		},
	})

	function isCurrent(current: LockSession) {
		return (
			mounted.value &&
			session.value === current &&
			projectId.value === current.projectId &&
			auth.value.user?.id === current.userId &&
			auth.value.token === current.token
		)
	}

	function assertCurrent(current: LockSession) {
		if (!isCurrent(current)) throw new ReviewLockError(formatMessage(messages.unverified))
	}

	function dismissTakeoverNotification(current: LockSession) {
		if (current.takeoverNotificationId === undefined) return
		removeNotification(current.takeoverNotificationId)
		current.takeoverNotificationId = undefined
	}

	function acceptStatus(current: LockSession, status: LockStatus, forSubmission = false) {
		if (status.locked && !status.expired && status.is_own_lock) {
			dismissTakeoverNotification(current)
			current.everOwned = true
			current.notifiedOwner = undefined
		} else if (status.locked && !status.expired) {
			const owner = status.locked_by?.id ?? ''
			if ((current.everOwned || forSubmission) && current.notifiedOwner !== owner) {
				dismissTakeoverNotification(current)
				current.notifiedOwner = owner
				current.takeoverNotificationId = addNotification({
					title: formatMessage(messages.takenOver, {
						username: status.locked_by?.username ?? formatMessage(messages.unknownModerator),
					}),
					type: 'warning',
					autoCloseMs: null,
					dismissible: true,
				}).id
			}
		} else {
			dismissTakeoverNotification(current)
			current.notifiedOwner = undefined
		}
	}

	const operation = useMutation({
		scope: { id: `project-review-lock-${instanceId}` },
		retry: false,
		onMutate: ({ session: current }: LockOperation) =>
			queryClient.cancelQueries({ queryKey: current.queryKey, exact: true }),
		mutationFn: async ({ session: current, kind }: LockOperation) => {
			if (kind === 'release') {
				await api.releaseLock(current.projectId, current.token)
				return
			}
			assertCurrent(current)
			if (kind === 'check') {
				const status = await api.checkLock(current.projectId)
				assertCurrent(current)
				return status
			}
			const response = await (kind === 'override'
				? api.overrideLock(current.projectId)
				: api.acquireLock(current.projectId))
			if (response.success && response.is_own_lock) current.everOwned = true
			assertCurrent(current)
			return { ...response, locked: true } satisfies LockStatus
		},
		onSuccess: (status, { session: current, forSubmission }) => {
			if (!status || !isCurrent(current)) return
			current.errorNotified = false
			acceptStatus(current, status, forSubmission)
			queryClient.setQueryData(current.queryKey, status)
		},
		onError: (error, { session: current, kind, forSubmission }) => {
			if (kind === 'release' || forSubmission || !isCurrent(current)) return
			if (current.errorNotified && kind !== 'override') return
			if (error instanceof ReviewLockError) return
			current.errorNotified = true
			const message =
				kind === 'override'
					? messages.takeOverFailed
					: kind === 'renew'
						? messages.renewFailed
						: messages.acquireFailed
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: formatMessage(message),
				type: 'error',
			})
		},
	})

	const statusQuery = useQuery({
		queryKey: computed(() => session.value?.queryKey ?? ['moderation', 'lock', instanceId]),
		enabled: computed(
			() =>
				mounted.value &&
				visibility.value === 'visible' &&
				!!session.value &&
				!operation.isPending.value,
		),
		queryFn: async ({ signal }) => {
			const current = session.value!
			assertCurrent(current)
			const status = await api.checkLock(current.projectId)
			signal.throwIfAborted()
			assertCurrent(current)
			acceptStatus(current, status)
			return status
		},
		staleTime: 30_000,
		gcTime: 0,
		refetchInterval: 30_000,
		refetchIntervalInBackground: false,
		refetchOnWindowFocus: false,
		retry: 1,
		retryDelay: 30_000,
	})
	const ownsLock = computed(
		() =>
			!!session.value &&
			isCurrent(session.value) &&
			!!statusQuery.data.value?.locked &&
			!statusQuery.data.value.expired &&
			statusQuery.data.value.is_own_lock,
	)
	const heldBy = computed<Labrinth.Moderation.Internal.LockedByUser | undefined>(() => {
		const status = statusQuery.data.value
		if (!session.value || !isCurrent(session.value) || !status?.locked || status.expired)
			return undefined
		return status.is_own_lock
			? undefined
			: (status.locked_by ?? { id: '', username: formatMessage(messages.unknownModerator) })
	})

	watch([statusQuery.data, statusQuery.dataUpdatedAt], ([status]) => {
		const current = session.value
		if (current && status && (!status.locked || status.expired) && !operation.isPending.value)
			void operation.mutateAsync({ session: current, kind: 'acquire' }).catch(() => undefined)
	})

	function endSession(current: LockSession) {
		dismissTakeoverNotification(current)
		if (session.value === current) session.value = undefined
		void operation.mutateAsync({ session: current, kind: 'release' }).catch(() => undefined)
	}

	function startSession() {
		if (session.value) endSession(session.value)
		if (!mounted.value || !projectId.value || !auth.value.token || !auth.value.user) return
		const current: LockSession = {
			projectId: projectId.value,
			userId: auth.value.user.id,
			token: auth.value.token,
			queryKey: ['moderation', 'lock', instanceId, projectId.value, ++generation],
			everOwned: false,
			errorNotified: false,
		}
		session.value = current
		void operation.mutateAsync({ session: current, kind: 'acquire' }).catch(() => undefined)
	}
	watch([projectId, () => auth.value.token, () => auth.value.user?.id, mounted], startSession, {
		flush: 'sync',
	})

	function checkOnFocus() {
		if (document.visibilityState === 'visible' && session.value && !operation.isPending.value)
			void statusQuery.refetch({ cancelRefetch: false })
	}

	function pageHide() {
		const current = session.value
		if (!current) return
		if (current.everOwned && auth.value.token === current.token) {
			const base = String(config.public.apiBaseUrl).replace(/\/v\d\/?$/, '/_internal/')
			navigator.sendBeacon(
				`${base}moderation/lock/${current.projectId}/release`,
				new Blob([current.token], { type: 'text/plain' }),
			)
		}
		endSession(current)
	}
	function pageShow(event: PageTransitionEvent) {
		if (event.persisted) startSession()
	}

	onMounted(() => {
		mounted.value = true
		renewalTimer = setInterval(() => {
			if (
				visibility.value === 'visible' &&
				session.value &&
				ownsLock.value &&
				!operation.isPending.value
			)
				void operation.mutateAsync({ session: session.value, kind: 'renew' }).catch(() => undefined)
		}, 5 * 60_000)
		window.addEventListener('focus', checkOnFocus)
		document.addEventListener('visibilitychange', checkOnFocus)
		window.addEventListener('pagehide', pageHide)
		window.addEventListener('pageshow', pageShow)
	})
	onScopeDispose(() => {
		mounted.value = false
		if (session.value) endSession(session.value)
		clearInterval(renewalTimer)
		if (import.meta.client) {
			window.removeEventListener('focus', checkOnFocus)
			document.removeEventListener('visibilitychange', checkOnFocus)
			window.removeEventListener('pagehide', pageHide)
			window.removeEventListener('pageshow', pageShow)
		}
	})

	function isEligibleStatus(status: LockStatus | undefined) {
		return !!status && (!status.locked || !!status.expired || status.is_own_lock)
	}

	async function ensureEligible(id: string) {
		const current = session.value
		if (!current || current.projectId !== id)
			throw new ReviewLockError(formatMessage(messages.unverified))
		let status: LockStatus | undefined
		try {
			status = await operation.mutateAsync({
				session: current,
				kind: 'check',
				forSubmission: true,
			})
		} catch {
			throw new ReviewLockError(formatMessage(messages.unverified))
		}
		assertCurrent(current)
		if (!isEligibleStatus(status)) throw new ReviewLockError(formatMessage(messages.unverified))
	}

	function assertEligible(id: string) {
		if (
			!session.value ||
			session.value.projectId !== id ||
			!isCurrent(session.value) ||
			!isEligibleStatus(statusQuery.data.value)
		)
			throw new ReviewLockError(formatMessage(messages.unverified))
	}

	return {
		heldBy,
		ownsLock,
		pending: operation.isPending,
		ensureEligible,
		assertEligible,
		takeOver: () => {
			if (!session.value || operation.isPending.value) return
			return operation
				.mutateAsync({ session: session.value, kind: 'override' })
				.catch(() => undefined)
		},
	}
}
