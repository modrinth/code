/**
 * Owyx friends presence heartbeat for the launcher sidebar.
 */

import { readonly, shallowRef } from 'vue'

import { type OwyxFriendPresence, postOwyxPresence } from '@/helpers/owyx-friends'

let heartbeatTimer: ReturnType<typeof setInterval> | null = null
let current: { status: OwyxFriendPresence; instanceName?: string | null } = {
	status: 'offline',
}

/** Live presence for UI (signed-in ≠ heartbeat running). */
const presenceStatus = shallowRef<OwyxFriendPresence>('offline')

export const owyxPresenceStatus = readonly(presenceStatus)

export function getOwyxPresenceStatus(): OwyxFriendPresence {
	return presenceStatus.value
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

export function startOwyxPresenceHeartbeat() {
	stopOwyxPresenceHeartbeat()
	setCurrent({ status: 'online', instanceName: null })
	void push()
	heartbeatTimer = setInterval(() => {
		void push()
	}, 30_000)
}

export function stopOwyxPresenceHeartbeat() {
	if (heartbeatTimer) {
		clearInterval(heartbeatTimer)
		heartbeatTimer = null
	}
	if (current.status !== 'offline') {
		setCurrent({ status: 'offline', instanceName: null })
		void push()
	}
}

export function setOwyxPresencePlaying(instanceName: string) {
	setCurrent({ status: 'playing', instanceName: instanceName.slice(0, 120) })
	void push()
}

export function setOwyxPresenceOnline() {
	setCurrent({ status: 'online', instanceName: null })
	void push()
}
