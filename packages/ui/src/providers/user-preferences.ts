import type { Labrinth } from '@modrinth/api-client'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type ComputedRef } from 'vue'

import { defineMessages, useVIntl } from '../composables/i18n'
import type { AuthProvider } from './auth'
import { createContext } from './create-context'
import type { AbstractWebNotificationManager } from './web-notifications'

export const userPreferencesQueryKey = (userId: string | null | undefined) =>
	['user', userId ?? null, 'preferences', 'v3'] as const

export interface UserPreferencesContext {
	preferences: ComputedRef<Labrinth.Users.v3.UserPreferences | undefined>
	isLoading: ComputedRef<boolean>
	isUpdating: ComputedRef<boolean>
	updatePreferences: (
		preferences: Labrinth.Users.v3.PartialUserPreferences,
	) => Promise<Labrinth.Users.v3.UserPreferences | undefined>
}

const [injectUserPreferencesContext, provideUserPreferencesContext] =
	createContext<UserPreferencesContext>('root', 'userPreferences')

export const injectUserPreferences = injectUserPreferencesContext

const messages = defineMessages({
	updateFailedTitle: {
		id: 'settings.preferences.update-failed.title',
		defaultMessage: 'Failed to sync settings',
	},
	updateFailedDescription: {
		id: 'settings.preferences.update-failed.description',
		defaultMessage: 'Your settings could not be saved to your Modrinth account. Please try again.',
	},
})

export function setupUserPreferencesProvider({
	auth,
	getPreferences,
	patchPreferences,
	notificationManager,
}: {
	auth: AuthProvider
	getPreferences: (userId: string) => Promise<Labrinth.Users.v3.UserPreferences>
	patchPreferences: (
		userId: string,
		preferences: Labrinth.Users.v3.PartialUserPreferences,
	) => Promise<Labrinth.Users.v3.UserPreferences>
	notificationManager: AbstractWebNotificationManager
}): UserPreferencesContext {
	const queryClient = useQueryClient()
	const { formatMessage } = useVIntl()
	const userId = computed(() => auth.user.value?.id ?? null)

	const preferencesQuery = useQuery({
		queryKey: computed(() => userPreferencesQueryKey(userId.value)),
		queryFn: () => {
			if (!userId.value) {
				throw new Error('A signed-in user is required to fetch preferences')
			}

			return getPreferences(userId.value)
		},
		enabled: computed(() => Boolean(userId.value)),
		staleTime: 30_000,
	})

	let updateQueue = Promise.resolve()
	const preferencesMutation = useMutation({
		mutationFn: ({
			userId,
			preferences,
		}: {
			userId: string
			preferences: Labrinth.Users.v3.PartialUserPreferences
		}) => {
			const request = updateQueue.then(() => patchPreferences(userId, preferences))
			updateQueue = request.then(
				() => undefined,
				() => undefined,
			)
			return request
		},
		onSuccess: (preferences, variables) => {
			queryClient.setQueryData(userPreferencesQueryKey(variables.userId), preferences)
		},
		onError: (_error, variables) => {
			void queryClient.invalidateQueries({
				queryKey: userPreferencesQueryKey(variables.userId),
			})
			notificationManager.addNotification({
				type: 'warning',
				title: formatMessage(messages.updateFailedTitle),
				text: formatMessage(messages.updateFailedDescription),
			})
		},
	})

	const context: UserPreferencesContext = {
		preferences: computed(() => preferencesQuery.data.value),
		isLoading: computed(() => preferencesQuery.isLoading.value),
		isUpdating: computed(() => preferencesMutation.isPending.value),
		updatePreferences: async (preferences) => {
			if (!userId.value) return undefined

			return preferencesMutation.mutateAsync({
				userId: userId.value,
				preferences,
			})
		},
	}

	provideUserPreferencesContext(context)
	return context
}
