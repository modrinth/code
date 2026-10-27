import { EyeIcon, EyeOffIcon } from '@modrinth/assets'
import {
	type ButtonMenuLeafOption,
	defineMessages,
	injectAuth,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import { libraryInstanceMessages } from '@/features/library/messages'
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
		networkMode: 'always' as const,
		mutationFn: ({ instanceId, hidden }: { instanceId: string; hidden: boolean }) =>
			setInstanceHidden(instanceId, hidden),
		onMutate: async ({ instanceId, hidden }) => {
			const key = hiddenInstanceKeys.ids(userId.value)
			await queryClient.cancelQueries({ queryKey: key })
			const previousIds = queryClient.getQueryData<string[]>(key)
			queryClient.setQueryData<string[]>(key, (ids = []) =>
				hidden
					? [instanceId, ...ids.filter((id) => id !== instanceId)]
					: ids.filter((id) => id !== instanceId),
			)
			return { key, previousIds }
		},
		onError: (error, _, context) => {
			if (context) queryClient.setQueryData(context.key, context.previousIds)
			handleError(error)
		},
		onSettled: () =>
			queryClient.invalidateQueries({ queryKey: hiddenInstanceKeys.ids(userId.value) }),
	})

	const setHidden = (instanceId: string, hidden: boolean) =>
		setHiddenMutation.mutateAsync({ instanceId, hidden })

	const hide = async (instanceId: string) => {
		if (!userId.value) return
		await setHidden(instanceId, true)
		addNotification({
			title: formatMessage(messages.hiddenTitle),
			text: formatMessage(messages.hiddenText),
			type: 'success',
		})
	}

	const show = (instanceId: string) => setHidden(instanceId, false)

	const canHide = computed(() => !!userId.value)

	const getVisibilityOption = (instanceId: string): ButtonMenuLeafOption =>
		hiddenIds.value.has(instanceId)
			? {
					id: 'show_in_library',
					label: formatMessage(libraryInstanceMessages.showInLibrary),
					icon: EyeIcon,
					shown: canHide.value,
					action: () => void show(instanceId),
				}
			: {
					id: 'hide',
					label: formatMessage(libraryInstanceMessages.hideFromLibrary),
					icon: EyeOffIcon,
					shown: canHide.value,
					action: () => void hide(instanceId),
				}

	return {
		hiddenIds,
		hiddenIdsLoaded: computed(() => hiddenQuery.isSuccess.value),
		canHide,
		setHidden,
		hide,
		show,
		getVisibilityOption,
	}
}
