import { nextTick, ref } from 'vue'

import { setPendingAuthMethod } from '@/composables/accounts.ts'
import { isAuthEmbedded } from '@/composables/auth-modal.ts'
import { type OAuthProviderId, oauthProviders } from '@/composables/oauth-providers.ts'

export const authProviderHandoff = ref(false)

let handoffToken: object | null = null

export function startAuthProviderHandoff(url: string) {
	authProviderHandoff.value = true
	const token = {}
	handoffToken = token

	void nextTick(() => {
		requestAnimationFrame(() => {
			requestAnimationFrame(() => {
				if (handoffToken !== token) {
					return
				}
				window.location.assign(url)
			})
		})
	})
}

export function cancelAuthProviderHandoff() {
	if (!authProviderHandoff.value) {
		return
	}
	authProviderHandoff.value = false
	handoffToken = null
	window.stop()
}

export function useProviderOpening() {
	const openingProviderName = ref<string | null>(null)
	const embedded = isAuthEmbedded()

	function beginProviderHandoff(
		provider: OAuthProviderId,
		event: MouseEvent,
		url: string,
		options?: { skip?: boolean; prepare?: () => void },
	) {
		setPendingAuthMethod(provider)
		options?.prepare?.()
		const name = oauthProviders.find((item) => item.id === provider)?.name
		if (!name || !embedded || options?.skip) {
			return
		}

		event.preventDefault()
		openingProviderName.value = name
		startAuthProviderHandoff(url)
	}

	function cancelProviderOpening() {
		cancelAuthProviderHandoff()
		openingProviderName.value = null
		setPendingAuthMethod(null)
	}

	return { openingProviderName, beginProviderHandoff, cancelProviderOpening }
}
