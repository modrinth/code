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
	const updatePreview = ref<
		Awaited<ReturnType<typeof install_get_shared_instance_update_preview>>
	>(null)
	const unavailableReason = ref<SharedInstanceUnavailableReason | null>(null)
	const availabilityCheckKey = ref<string | null>(null)
	const availabilityRefresh = ref(0)
	let availabilityRequestId = 0

	const expectedUserId = computed(() => instance.value?.shared_instance?.linked_user_id ?? null)
	const wrongAccount = computed(() => {
		if (auth.isReady && !auth.isReady.value) return false
		if (!expectedUserId.value) return false
		return auth.user.value?.id !== expectedUserId.value
	})
	const actionsLocked = computed(() => wrongAccount.value)
	const shareActionsLocked = computed(
		() => actionsLocked.value || unavailableReason.value !== null,
	)
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
		availabilityCheckKey.value = null
		updatePreview.value = null
		unavailableReason.value = null
	}

	function refreshAvailability() {
		availabilityRefresh.value++
	}

	function setUnavailable(reason: SharedInstanceUnavailableReason | null) {
		availabilityRequestId++
		availabilityCheckKey.value = null
		updatePreview.value = null
		unavailableReason.value = reason
	}

	async function checkAvailability(instanceId: string, key: string) {
		const requestId = ++availabilityRequestId
		try {
			const preview = await install_get_shared_instance_update_preview(instanceId)
			if (!isCurrentRequest(requestId, instanceId, key)) return
			updatePreview.value = preview
			unavailableReason.value = null
		} catch (error) {
			if (!isCurrentRequest(requestId, instanceId, key)) return
			if (isSharedInstanceUnavailableError(error)) {
				updatePreview.value = null
				unavailableReason.value = getSharedInstanceUnavailableReason(error)
				return
			}
			notifyError(error)
		}
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
				availabilityCheckKey.value = null
				updatePreview.value = null
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
