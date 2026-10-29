import type { ServerSharingUpdateModal } from '@modrinth/ui'

const SHOWN_KEY = 'server-sharing-update-shown'

/**
 * Shows the server sharing update modal once per browser, the first time a server flagged
 * for the play migration is opened.
 */
export function useServerSharingUpdate(
	modal: Ref<InstanceType<typeof ServerSharingUpdateModal> | null>,
	serverId: string,
) {
	const pending = ref(false)

	watch([pending, modal], ([isPending, instance]) => {
		if (!isPending || !instance) return
		pending.value = false
		try {
			if (localStorage.getItem(SHOWN_KEY) === 'true') return
			localStorage.setItem(SHOWN_KEY, 'true')
		} catch {
			return
		}
		instance.show({ path: `/hosting/manage/${serverId}/play`, playPage: true })
	})

	return {
		show: () => {
			pending.value = true
		},
	}
}
