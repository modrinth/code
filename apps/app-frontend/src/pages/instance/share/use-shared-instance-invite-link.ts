import type { InviteLinkSettings } from '@modrinth/ui'
import { computed, type Ref, ref, watch } from 'vue'

import { config } from '@/config'
import { toError } from '@/helpers/errors'
import { create_shared_instance_invite_link } from '@/helpers/instance'

const DEFAULT_INVITE_LINK_MAX_USES = 10

export function useSharedInstanceInviteLink(
	instanceId: Ref<string>,
	maxInviteUses: Ref<number>,
	onError: (error: unknown) => void,
) {
	const details = ref<Awaited<ReturnType<typeof create_shared_instance_invite_link>>>()
	const pending = ref(false)
	const link = computed(() =>
		details.value
			? `${config.siteUrl}/share/${encodeURIComponent(details.value.inviteId)}`
			: undefined,
	)

	async function ensure() {
		if (details.value) return true
		if (pending.value) return false
		const maxUses = Math.min(DEFAULT_INVITE_LINK_MAX_USES, Math.floor(maxInviteUses.value))
		if (maxUses <= 0) return false

		pending.value = true
		try {
			details.value = await create_shared_instance_invite_link(instanceId.value, { maxUses })
			return true
		} catch (error) {
			onError(error)
			return false
		} finally {
			pending.value = false
		}
	}

	async function update(settings: InviteLinkSettings) {
		if (!details.value) return
		const maxInviteLinkUses = Math.floor(maxInviteUses.value)
		if (maxInviteLinkUses <= 0) return

		pending.value = true
		try {
			const maxAgeSeconds = Math.max(
				1,
				Math.min(604800, Math.floor((settings.expiresAt.getTime() - Date.now()) / 1000)),
			)
			details.value = await create_shared_instance_invite_link(instanceId.value, {
				maxAgeSeconds,
				maxUses: Math.min(settings.maxUses, maxInviteLinkUses),
				replaceInviteId: details.value.inviteId,
			})
		} catch (error) {
			throw toError(error)
		} finally {
			pending.value = false
		}
	}

	watch(instanceId, () => {
		details.value = undefined
		pending.value = false
	})

	return { details, pending, link, ensure, update }
}
