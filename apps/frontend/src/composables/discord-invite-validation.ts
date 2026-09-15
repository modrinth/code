import { ModrinthApiError } from '@modrinth/api-client'
import { defineMessages, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { refDebounced, useMounted } from '@vueuse/core'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { normalizeProjectUrl } from '~/helpers/project-url'

const DISCORD_API = 'https://discord.com/api'
const DISCORD_INVITE_DOMAINS = ['discord.com', 'discordapp.com']
const DISCORD_SHORT_INVITE_DOMAIN = 'discord.gg'

const messages = defineMessages({
	invalid: {
		id: 'project.settings.links.discord-invite-invalid',
		defaultMessage: 'This Discord invite is invalid or expired. Replace it with an active server invite.',
	},
})

function inviteCode(value: string): string | undefined {
	try {
		const url = new URL(normalizeProjectUrl(value))
		if (url.protocol !== 'https:' || url.username || url.password) return
		const host = url.hostname.replace(/\.$/, '')
		const matchesDomain = (domain: string) => host === domain || host.endsWith(`.${domain}`)
		const parts = url.pathname.split('/').filter(Boolean)
		const code =
			matchesDomain(DISCORD_SHORT_INVITE_DOMAIN) && parts.length === 1
				? parts[0]
				: DISCORD_INVITE_DOMAINS.some(matchesDomain) &&
					  parts.length === 2 &&
					  parts[0] === 'invite'
					? parts[1]
					: undefined
		return code && /^[a-zA-Z0-9_-]+$/.test(code) ? code : undefined
	} catch {
		return undefined
	}
}

export function useDiscordInviteValidation(value: MaybeRefOrGetter<string>) {
	const client = injectModrinthClient()
	const mounted = useMounted()
	const code = computed(() => inviteCode(toValue(value)))
	const debouncedCode = refDebounced(code, 500)
	const { data } = useQuery({
		queryKey: computed(() => ['discord', 'invite', debouncedCode.value]),
		enabled: computed(
			() => mounted.value && !!code.value && code.value === debouncedCode.value,
		),
		queryFn: async ({ signal }) => {
			try {
				const invite = await client.request<Record<string, unknown>>(
					`/invites/${encodeURIComponent(debouncedCode.value!)}`,
					{
						api: DISCORD_API,
						version: 10,
						skipAuth: true,
						headers: { 'Content-Type': '', Accept: 'application/json' },
						retry: false,
						timeout: 5000,
						signal,
					},
				)
				return (
					!invite.guild ||
					(typeof invite.expires_at === 'string' && Date.parse(invite.expires_at) < Date.now())
				)
			} catch (error) {
				if (error instanceof ModrinthApiError && error.statusCode === 404) return true
				throw error
			}
		},
		staleTime: 60_000,
		retry: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
	})

	return computed(() =>
		code.value && code.value === debouncedCode.value && data.value === true
			? { severity: 'error' as const, message: messages.invalid }
			: null,
	)
}
