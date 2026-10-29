import { ref, watch } from 'vue'

import { injectModrinthClient } from '#ui/providers'

import { createServerInviteLink } from './invite-link'

export type ServerInviteLinkTarget = { serverId: string; worldId: string; siteUrl: string }

export type ServerInviteLink = ReturnType<typeof useServerInviteLink>

/** Loads a share URL for a server world, with retry and copied state for the invite link UI. */
export function useServerInviteLink() {
	const client = injectModrinthClient()
	const link = ref<string | null>(null)
	const loading = ref(false)
	const error = ref<string | null>(null)
	const copied = ref(false)
	let run = 0
	let target: ServerInviteLinkTarget | null = null

	watch(link, () => {
		copied.value = false
	})

	async function retry() {
		if (!target || loading.value) return
		const current = ++run
		const { serverId, worldId, siteUrl } = target
		loading.value = true
		error.value = null
		try {
			const next = await createServerInviteLink(
				client,
				serverId,
				worldId,
				siteUrl,
				() => current === run,
			)
			if (current === run) link.value = next
		} catch (err) {
			if (current === run) error.value = err instanceof Error ? err.message : String(err)
		} finally {
			if (current === run) loading.value = false
		}
	}

	function start(next: ServerInviteLinkTarget) {
		run++
		target = next
		link.value = null
		loading.value = false
		error.value = null
		copied.value = false
		void retry()
	}

	/** Stops an in-flight load and keeps whatever link is already shown. */
	function cancel() {
		run++
		loading.value = false
	}

	function reset() {
		cancel()
		target = null
		link.value = null
		error.value = null
		copied.value = false
	}

	return { link, loading, error, copied, start, retry, cancel, reset }
}
