import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

import { projectQueryOptions } from '~/composables/queries/project'

export function useThreadProjectOwner(
	thread: Ref<{
		project_id?: string | null
		messages: { body: { type: string } }[]
	}>,
) {
	const client = injectModrinthClient()
	const enabled = computed(
		() =>
			!!thread.value.project_id &&
			thread.value.messages.some((message) => message.body.type === 'auto_approval'),
	)
	const { data: members } = useQuery(
		computed(() => ({
			...projectQueryOptions.members(thread.value.project_id ?? '', client),
			enabled: enabled.value,
		})),
	)
	const owner = computed(() => members.value?.find((member) => member.is_owner)?.user)
	const { data: organization } = useQuery(
		computed(() => ({
			...projectQueryOptions.organization(thread.value.project_id ?? '', client),
			enabled: enabled.value && !!members.value && !owner.value,
		})),
	)
	return computed<Labrinth.Users.v3.User | undefined>(
		() => owner.value ?? organization.value?.members.find((member) => member.is_owner)?.user,
	)
}
