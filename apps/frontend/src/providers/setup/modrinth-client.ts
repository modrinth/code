import { ModrinthServerError } from '@modrinth/api-client'
import {
	type AbstractWebNotificationManager,
	defineMessages,
	provideModrinthClient,
	useVIntl,
} from '@modrinth/ui'

import { createModrinthClient } from '~/helpers/api.ts'

const messages = defineMessages({
	permissionRemovedTitle: {
		id: 'error.permission-removed.title',
		defaultMessage: 'Action not allowed',
	},
	permissionRemovedText: {
		id: 'error.permission-removed.text',
		defaultMessage:
			'A moderator has restricted this action on your account. See Account standing in your account settings for details.',
	},
})

export function setupModrinthClientProvider(
	auth: Awaited<ReturnType<typeof useAuth>>,
	notificationManager: AbstractWebNotificationManager,
) {
	const config = useRuntimeConfig()
	const { formatMessage } = useVIntl()
	const client = createModrinthClient(auth, {
		apiBaseUrl: config.public.apiBaseUrl.replace('/v2/', '/'),
		archonBaseUrl: config.public.pyroBaseUrl.replace('/v2/', '/'),
		sharedInstancesBaseUrl: config.public.sharedInstancesBaseUrl,
		commitHash: config.public.hash,
		rateLimitKey: config.rateLimitKey,
		onError: (error) => {
			if (
				import.meta.client &&
				error instanceof ModrinthServerError &&
				error.v1Error?.error === 'permission_removed'
			) {
				notificationManager.addNotification({
					type: 'error',
					title: formatMessage(messages.permissionRemovedTitle),
					text: formatMessage(messages.permissionRemovedText),
				})
			}
		},
	})
	provideModrinthClient(client)
	return client
}
