import { watch } from 'vue'

import { hydrateStoredAccounts, isSessionYoung, useStoredAccounts } from '@/composables/accounts.ts'
import { useAuthState } from '@/composables/auth.ts'
import { authModalLocation, promptAuth } from '@/composables/auth-modal.ts'
import { isPrivilegedTokenExpired } from '@/composables/privileged-session.ts'

export class ReauthCancelled extends Error {
	constructor() {
		super('Reauthentication cancelled')
		this.name = 'ReauthCancelled'
	}
}

export function isReauthCancelled(error: unknown): boolean {
	return error instanceof ReauthCancelled
}

let pending: Promise<void> | null = null

export function requestReauth(): Promise<void> {
	if (!pending) {
		pending = openReauth().finally(() => {
			pending = null
		})
	}
	return pending
}

async function openReauth(): Promise<void> {
	const before = useAuthState().value.token
	hydrateStoredAccounts()
	const userId = useAuthState().value.user?.id
	const account = useStoredAccounts().value.find((item) => item.id === userId)
	const route = useRoute()

	await promptAuth({
		path: '/auth/reauthenticate',
		query: {
			account: account?.id ?? userId,
			redirect: route.fullPath,
		},
	})

	if (authModalLocation.value?.path !== '/auth/reauthenticate') {
		throw new ReauthCancelled()
	}

	const next = await new Promise<{ token: string; userId?: string }>((resolve) => {
		const stop = watch(authModalLocation, (location) => {
			if (location) {
				return
			}
			stop()
			const auth = useAuthState().value
			resolve({ token: auth.token, userId: auth.user?.id })
		})
	})

	const sameUser = Boolean(userId) && next.userId === userId
	if (!sameUser || !next.token || next.token === before || isSessionYoung(next.token) === false) {
		throw new ReauthCancelled()
	}
}

export async function runPrivileged(action: () => Promise<void>, precheck = true): Promise<void> {
	if (precheck && isSessionYoung(useAuthState().value.token) === false) {
		await requestReauth()
	}

	try {
		await action()
	} catch (error) {
		if (!isPrivilegedTokenExpired(error)) {
			throw error
		}
		await requestReauth()
		await action()
	}
}
