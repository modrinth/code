import { defineMessages, injectAuth, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import { setInstanceHidden } from '@/platform/app-lib/hidden-instances/commands'

import { hiddenInstanceIdsQueryOptions, hiddenInstanceKeys } from './queries'

const messages = defineMessages({
	hiddenTitle: {
		id: 'app.library.instance-hidden.title',
		defaultMessage: 'Hidden from library',
	},
	hiddenText: {
		id: 'app.library.instance-hidden.text',
		defaultMessage: 'You can show it again in Settings > Behavior.',
	},
})

export function useHiddenInstances() {
	const auth = injectAuth()
	const queryClient = useQueryClient()
	const { handleError, addNotification } = injectNotificationManager()
	const { formatMessage } = useVIntl()
	const userId = computed(() => auth.user.value?.id ?? null)

	const hiddenQuery = useQuery(
		computed(() => ({
			...hiddenInstanceIdsQueryOptions(userId.value),
			enabled: !!userId.value,
		})),
	)
	const hiddenIds = computed(() => new Set(hiddenQuery.data.value ?? []))

	const setHiddenMutation = useMutation({
		mutationFn: ({ instanceId, hidden }: { instanceId: string; hidden: boolean }) =>
			setInstanceHidden(instanceId, hidden),
		onMutate: ({ instanceId, hidden }) => {
			queryClient.setQueryData<string[]>(hiddenInstanceKeys.ids(userId.value), (ids = []) =>
				hidden
					? [instanceId, ...ids.filter((id) => id !== instanceId)]
					: ids.filter((id) => id !== instanceId),
			)
		},
		onError: handleError,
		onSettled: () =>
			queryClient.invalidateQueries({ queryKey: hiddenInstanceKeys.ids(userId.value) }),
	})

	const setHidden = (instanceId: string, hidden: boolean) =>
		setHiddenMutation.mutateAsync({ instanceId, hidden })

	const hide = async (instanceId: string) => {
		await setHidden(instanceId, true)
		addNotification({
			title: formatMessage(messages.hiddenTitle),
			text: formatMessage(messages.hiddenText),
			type: 'success',
		})
	}

	const show = (instanceId: string) => setHidden(instanceId, false)

	return {
		hiddenIds,
		hiddenIdsLoaded: computed(() => hiddenQuery.isSuccess.value),
		canHide: computed(() => !!userId.value),
		setHidden,
		hide,
		show,
	}
}
