/**
 * Owyx friends presence heartbeat for the launcher sidebar.
 */

import { type OwyxFriendPresence, postOwyxPresence } from '@/helpers/owyx-friends'

let heartbeatTimer: ReturnType<typeof setInterval> | null = null
let current: { status: OwyxFriendPresence; instanceName?: string | null } = {
	status: 'offline',
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
	current = { status: 'online', instanceName: null }
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
		current = { status: 'offline', instanceName: null }
		void push()
	}
}

export function setOwyxPresencePlaying(instanceName: string) {
	current = { status: 'playing', instanceName: instanceName.slice(0, 120) }
	void push()
}

export function setOwyxPresenceOnline() {
	current = { status: 'online', instanceName: null }
	void push()
}
