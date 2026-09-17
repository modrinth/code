/**
 * Owyx friends presence heartbeat for the launcher sidebar.
 * Respects sharePresence from /api/friends/settings — when off, no heartbeats
 * and friends see offline.
 */

import { readonly, shallowRef } from 'vue'

import { type OwyxFriendPresence, postOwyxPresence } from '@/helpers/owyx-friends'

let heartbeatTimer: ReturnType<typeof setInterval> | null = null
let sharePresenceEnabled = true
let current: { status: OwyxFriendPresence; instanceName?: string | null } = {
	status: 'offline',
}

/** Live presence for UI (signed-in ≠ heartbeat running). */
const presenceStatus = shallowRef<OwyxFriendPresence>('offline')

export const owyxPresenceStatus = readonly(presenceStatus)

export function getOwyxPresenceStatus(): OwyxFriendPresence {
	return presenceStatus.value
}

export function isOwyxSharePresenceEnabled(): boolean {
	return sharePresenceEnabled
}

function setCurrent(next: { status: OwyxFriendPresence; instanceName?: string | null }) {
	current = next
	presenceStatus.value = next.status
}

async function push() {
	try {
		await postOwyxPresence(current)
	} catch {
		/* best-effort */
	}
}

function clearHeartbeatTimer() {
	if (heartbeatTimer) {
		clearInterval(heartbeatTimer)
		heartbeatTimer = null
	}
}

/**
 * Apply sharePresence from social settings.
 * OFF → stop heartbeats and force offline to friends.
 * ON → start heartbeat if not already running.
 */
export function setOwyxSharePresenceEnabled(enabled: boolean) {
	sharePresenceEnabled = enabled
	if (enabled) {
		startOwyxPresenceHeartbeat()
	} else {
		stopOwyxPresenceHeartbeat({ forceOfflinePush: true })
	}
}

export function startOwyxPresenceHeartbeat() {
	clearHeartbeatTimer()
	if (!sharePresenceEnabled) {
		setCurrent({ status: 'offline', instanceName: null })
		return
	}
	setCurrent({ status: 'online', instanceName: null })
	void push()
	heartbeatTimer = setInterval(() => {
		void push()
	}, 30_000)
}

export function stopOwyxPresenceHeartbeat(opts?: { forceOfflinePush?: boolean }) {
	clearHeartbeatTimer()
	const wasLive = current.status !== 'offline'
	setCurrent({ status: 'offline', instanceName: null })
	if (wasLive || opts?.forceOfflinePush) {
		void push()
	}
}

export function setOwyxPresencePlaying(instanceName: string) {
	if (!sharePresenceEnabled) return
	setCurrent({ status: 'playing', instanceName: instanceName.slice(0, 120) })
	void push()
}

export function setOwyxPresenceOnline() {
	if (!sharePresenceEnabled) return
	setCurrent({ status: 'online', instanceName: null })
	void push()
}
