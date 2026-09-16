import { createContext } from '@modrinth/ui'
import { computed, type ComputedRef, type Ref } from 'vue'

import type { OwyxSiteSession } from '@/helpers/owyx-site-auth'

export type OwyxSiteSessionContext = {
	session: Ref<OwyxSiteSession | null | undefined>
	isSignedIn: ComputedRef<boolean>
	isStaff: ComputedRef<boolean>
	isAdmin: ComputedRef<boolean>
	refresh: () => Promise<void>
	signIn: () => Promise<void>
	signOut: () => Promise<void>
}

export const [injectOwyxSiteSession, provideOwyxSiteSession] =
	createContext<OwyxSiteSessionContext>('root', 'owyxSiteSession')

export function setupOwyxSiteSessionProvider(
	session: Ref<OwyxSiteSession | null | undefined>,
	refresh: () => Promise<void>,
	signIn: () => Promise<void>,
	signOut: () => Promise<void>,
): OwyxSiteSessionContext {
	const context: OwyxSiteSessionContext = {
		session,
		isSignedIn: computed(() => !!session.value?.token && !!session.value?.user),
		isStaff: computed(() => {
			const role = session.value?.user?.role
			return role === 'admin' || role === 'moderator'
		}),
		isAdmin: computed(() => session.value?.user?.role === 'admin'),
		refresh,
		signIn,
		signOut,
	}
	provideOwyxSiteSession(context)
	return context
}
