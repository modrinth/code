import { injectAuth } from '@modrinth/ui'
import { computed, inject, type InjectionKey, provide, type Ref, ref, watch } from 'vue'

import { useUserQuery } from '@/composables/users/use-user-query'
import {
	getSharedInstanceUnavailableReason,
	install_get_shared_instance_update_preview,
	isSharedInstanceUnavailableError,
	type SharedInstanceUnavailableReason,
} from '@/helpers/install'
import type { GameInstance } from '@/helpers/types'

export type SharedInstanceManager =
	| {
			type: 'user'
			name: string
			avatarUrl?: string
			tintBy: string
	  }
	| {
			type: 'server'
			name: string
			avatarUrl?: string
			tintBy: string
	  }

export function useSharedInstanceState(
	instance: Ref<GameInstance | undefined>,
	offline: Ref<boolean>,
	notifyError: (error: unknown) => void,
) {
	const auth = injectAuth()
	const updatePreview =
		ref<Awaited<ReturnType<typeof install_get_shared_instance_update_preview>>>(null)
	const updatePreviewLoaded = ref(false)
	const unavailableReason = ref<SharedInstanceUnavailableReason | null>(null)
	const availabilityCheckKey = ref<string | null>(null)
	const availabilityRefresh = ref(0)
	let availabilityRequestId = 0
	let availabilityRequest: {
		key: string
		promise: Promise<{
			preview: Awaited<ReturnType<typeof install_get_shared_instance_update_preview>>
			error: unknown | null
		}>
	} | null = null

	const expectedUserId = computed(() => instance.value?.shared_instance?.linked_user_id ?? null)
	const wrongAccount = computed(() => {
		if (auth.isReady && !auth.isReady.value) return false
		if (!expectedUserId.value) return false
		return auth.user.value?.id !== expectedUserId.value
	})
	const actionsLocked = computed(() => wrongAccount.value)
	const shareActionsLocked = computed(() => actionsLocked.value || unavailableReason.value !== null)
	const signedOut = computed(() => !auth.session_token.value)
	const managerUserId = computed(() => {
		const attachment = instance.value?.shared_instance
		if (!attachment) return null
		if (attachment.role === 'owner') {
			return actionsLocked.value ? (attachment.linked_user_id ?? null) : null
		}
		return attachment.manager_id ?? null
	})
	const managerUserQuery = useUserQuery(managerUserId)
	const manager = computed<SharedInstanceManager | null>(() => {
		const attachment = instance.value?.shared_instance
		if (!attachment) return null

		if (attachment.server_manager_name) {
			return {
				type: 'server',
				name: attachment.server_manager_name,
				avatarUrl: attachment.server_manager_icon_url ?? undefined,
				tintBy: attachment.server_manager_name,
			}
		}

		const user = managerUserQuery.data.value
		if (!user) return null
		return {
			type: 'user',
			name: user.username,
			avatarUrl: user.avatar_url ?? undefined,
			tintBy: user.id,
		}
	})
	const unavailableManager = computed(() => manager.value?.name ?? null)

	function reset() {
		availabilityRequestId++
		availabilityRequest = null
		availabilityCheckKey.value = null
		updatePreview.value = null
		updatePreviewLoaded.value = false
		unavailableReason.value = null
	}

	function refreshAvailability() {
		availabilityCheckKey.value = null
		updatePreviewLoaded.value = false
		availabilityRefresh.value++
	}

	function setUnavailable(reason: SharedInstanceUnavailableReason | null) {
		availabilityRequestId++
		availabilityRequest = null
		availabilityCheckKey.value = null
		updatePreview.value = null
		updatePreviewLoaded.value = false
		unavailableReason.value = reason
	}

	async function checkAvailability(instanceId: string, key: string, throwError = false) {
		const requestId = ++availabilityRequestId
		let request = availabilityRequest
		if (!request || request.key !== key) {
			const promise = install_get_shared_instance_update_preview(instanceId).then(
				(preview) => ({ preview, error: null }),
				(error: unknown) => ({ preview: null, error }),
			)
			request = { key, promise }
			availabilityRequest = request
			void promise.finally(() => {
				if (availabilityRequest?.promise === promise) availabilityRequest = null
			})
		}

		const result = await request.promise
		if (!isCurrentRequest(requestId, instanceId, key)) return null

		if (result.error !== null) {
			updatePreviewLoaded.value = false
			if (isSharedInstanceUnavailableError(result.error)) {
				updatePreview.value = null
				unavailableReason.value = getSharedInstanceUnavailableReason(result.error)
			} else if (!throwError) {
				notifyError(result.error)
			}

			if (throwError) throw result.error
			return null
		}

		updatePreview.value = result.preview
		updatePreviewLoaded.value = true
		unavailableReason.value = null
		return result.preview
	}

	async function refreshUpdatePreview() {
		const instanceId = instance.value?.id
		const userId = auth.user.value?.id
		if (!instanceId || !userId) return null

		const key = `${instanceId}:${userId}`
		if (availabilityCheckKey.value === key && updatePreviewLoaded.value) {
			return updatePreview.value
		}

		availabilityCheckKey.value = key
		return await checkAvailability(instanceId, key, true)
	}

	function isCurrentRequest(requestId: number, instanceId: string, key: string) {
		return (
			requestId === availabilityRequestId &&
			instance.value?.id === instanceId &&
			availabilityCheckKey.value === key
		)
	}

	watch(
		() => ({
			refresh: availabilityRefresh.value,
			instanceId: instance.value?.id,
			role: instance.value?.shared_instance?.role,
			locked: actionsLocked.value,
			offline: offline.value,
			signedIn: !!auth.session_token.value,
			userId: auth.user.value?.id ?? null,
			authReady: auth.isReady?.value ?? true,
		}),
		async ({ instanceId, role, locked, offline, signedIn, userId, authReady }) => {
			if (
				!instanceId ||
				role !== 'member' ||
				locked ||
				offline ||
				!authReady ||
				!signedIn ||
				!userId
			) {
				availabilityRequestId++
				availabilityRequest = null
				availabilityCheckKey.value = null
				updatePreview.value = null
				updatePreviewLoaded.value = false
				if (instanceId && role) unavailableReason.value = null
				return
			}

			const key = `${instanceId}:${userId}`
			if (availabilityCheckKey.value === key) return
			availabilityCheckKey.value = key
			await checkAvailability(instanceId, key)
		},
		{ immediate: true },
	)

	return {
		actionsLocked,
		shareActionsLocked,
		unavailableReason,
		unavailableManager,
		manager,
		updatePreview,
		expectedUserId,
		wrongAccount,
		signedOut,
		reset,
		refreshAvailability,
		refreshUpdatePreview,
		setUnavailable,
	}
}

export type SharedInstanceState = ReturnType<typeof useSharedInstanceState>

const sharedInstanceStateKey: InjectionKey<SharedInstanceState> = Symbol('shared-instance-state')

export function provideSharedInstanceState(state: SharedInstanceState) {
	provide(sharedInstanceStateKey, state)
}

export function injectSharedInstanceState() {
	const state = inject(sharedInstanceStateKey)
	if (!state) throw new Error('Shared instance state has not been provided.')
	return state
}
