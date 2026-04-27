<template>
	<div
		class="universal-card flex w-full max-w-[28rem] flex-col gap-6 border border-solid border-surface-5"
	>
		<div v-if="authorizationError" class="flex flex-col gap-2.5">
			<h1 class="m-0 mx-auto text-xl font-semibold text-contrast">
				{{ formatMessage(messages.errorTitle) }}
			</h1>
			<p class="m-0">
				<span>{{ authorizationError.error }}: </span>
				{{ authorizationError.description }}
			</p>
		</div>
		<div v-else-if="app && createdBy && authorizationData" class="flex flex-col gap-6">
			<div class="mt-4 flex items-center justify-center">
				<div class="flex w-full flex-row items-center justify-evenly">
					<Avatar size="md" :src="app.icon_url" />
					<!-- <img class="profile-pic" :src="app.icon_url" alt="User profile picture" /> -->
					<div class="flex select-none items-center justify-center text-[2rem] text-primary">→</div>
					<Avatar size="md" circle :src="auth.user?.avatar_url" />
					<!-- <img class="profile-pic" :src="auth.user.avatar_url" alt="User profile picture" /> -->
				</div>
			</div>
			<div class="mx-auto">
				<h1 class="mb-0 ml-0 mr-0 mt-0 text-xl text-contrast">
					{{ formatMessage(messages.title, { appName: app.name }) }}
				</h1>
			</div>
			<div class="flex flex-col gap-3">
				<div class="mb-3">
					<IntlFormatted
						:message-id="messages.appInfo"
						:values="{
							appName: app.name,
							creator: createdBy.username,
						}"
					>
						<template #strong="{ children }">
							<strong>
								<component :is="() => normalizeChildren(children)" />
							</strong>
						</template>
						<template #creator-link="{ children }">
							<nuxt-link class="text-link" :to="'/user/' + createdBy.id">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</div>
				<div class="flex flex-col gap-3">
					<div v-for="scopeItem in scopeDefinitions" :key="scopeItem">
						<div class="flex flex-row items-center gap-2">
							<div class="grid h-5 min-h-5 w-5 min-w-5 place-content-center rounded-full bg-green">
								<CheckIcon class="text-sm text-black" />
							</div>
							{{ scopeItem }}
						</div>
					</div>
				</div>
			</div>
			<div class="flex flex-row justify-center gap-2">
				<Button class="!w-full" large :action="onReject" :disabled="pending">
					<XIcon />
					{{ formatMessage(messages.decline) }}
				</Button>
				<Button class="!w-full" color="primary" large :action="onAuthorize" :disabled="pending">
					<CheckIcon />
					{{ formatMessage(messages.authorize) }}
				</Button>
			</div>
			<div class="flex flex-col gap-2 text-center">
				<p class="m-0 text-sm">
					<IntlFormatted :message-id="messages.redirectUrl" :values="{ url: redirectUri }">
						<template #redirect-url="{ children }">
							<span class="font-bold">
								<component :is="() => normalizeChildren(children)" />
							</span>
						</template>
					</IntlFormatted>
				</p>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon, XIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	IntlFormatted,
	normalizeChildren,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'
import type { LocationQueryValue } from 'vue-router'

import { useScopes } from '@/composables/auth/scopes.ts'

interface ApiErrorShape {
	data?: {
		description?: string
		error?: string
	}
}

const getQueryString = (
	value: LocationQueryValue | LocationQueryValue[] | null | undefined,
): string => {
	const firstValue = Array.isArray(value) ? value[0] : value
	return typeof firstValue === 'string' ? firstValue : ''
}

const getErrorMessage = (error: unknown): string => {
	const apiError = error as ApiErrorShape
	if (typeof apiError?.data?.description === 'string') {
		return apiError.data.description
	}
	if (error instanceof Error) {
		return error.message
	}
	return String(error)
}

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	appInfo: {
		id: 'auth.authorize.app-info',
		defaultMessage:
			'<strong>{appName}</strong> by <creator-link>{creator}</creator-link> will be able to:',
	},
	authorize: {
		id: 'auth.authorize.action.authorize',
		defaultMessage: 'Authorize',
	},
	decline: {
		id: 'auth.authorize.action.decline',
		defaultMessage: 'Decline',
	},
	noRedirectUrlError: {
		id: 'auth.authorize.error.no-redirect-url',
		defaultMessage: 'No redirect location found in response',
	},
	missingParametersError: {
		id: 'auth.authorize.error.missing-parameters',
		defaultMessage: 'Missing required OAuth query parameters.',
	},
	redirectUrl: {
		id: 'auth.authorize.redirect-url',
		defaultMessage: 'You will be redirected to <redirect-url>{url}</redirect-url>',
	},
	title: {
		id: 'auth.authorize.authorize-app-name',
		defaultMessage: 'Authorize {appName}',
	},
	errorTitle: {
		id: 'auth.authorize.errro-title',
		defaultMessage: 'An Error Occured',
	},
})

const router = useNativeRoute()
const auth = await useAuth()
const { scopesToDefinitions } = useScopes()

const clientId = computed(() => getQueryString(router.query.client_id))
const redirectUri = computed(() => getQueryString(router.query.redirect_uri))
const scope = computed(() => getQueryString(router.query.scope))
const state = computed(() => getQueryString(router.query.state))
const hasRequiredParams = computed(() => !!clientId.value && !!redirectUri.value && !!scope.value)

const getFlowIdAuthorization = async () => {
	const authorization = await client.labrinth.oauth_internal.authorize({
		client_id: clientId.value,
		redirect_uri: redirectUri.value,
		scope: scope.value,
		...(state.value ? { state: state.value } : {}),
	})

	if (typeof authorization === 'string') {
		await navigateTo(authorization, {
			external: true,
		})
		return null
	}

	return authorization
}

const {
	data: authorizationData,
	isPending: pending,
	error,
	suspense: authSusp,
} = useQuery({
	queryKey: computed(() => [
		'authorization',
		clientId.value,
		redirectUri.value,
		scope.value,
		state.value,
	]),
	queryFn: getFlowIdAuthorization,
	enabled: hasRequiredParams,
})

const authorizationError = computed(() => {
	if (!hasRequiredParams.value) {
		return {
			error: 'invalid_request',
			description: formatMessage(messages.missingParametersError),
		}
	}

	const apiError = (error.value as ApiErrorShape | null)?.data
	if (apiError) {
		return apiError
	}

	if (error.value) {
		return {
			error: 'server_error',
			description: getErrorMessage(error.value),
		}
	}

	return null
})

const { data: app, suspense: appSusp } = useQuery({
	queryKey: computed(() => ['oauth/app', clientId.value]),
	queryFn: () => client.labrinth.oauth_internal.getApp(clientId.value),
	enabled: computed(() => !!clientId.value),
})

const { data: createdBy, suspense: userSusp } = useQuery({
	queryKey: computed(() => ['user', app.value?.created_by]),
	queryFn: () => {
		if (!app.value?.created_by) {
			throw new Error('Missing OAuth app creator')
		}

		return client.labrinth.users_v2.get(app.value.created_by)
	},
	enabled: computed(() => !!app.value?.created_by),
})

onServerPrefetch(async () => {
	await Promise.allSettled([authSusp(), appSusp(), userSusp()])
})

const scopeDefinitions = computed(() =>
	scopesToDefinitions(BigInt(authorizationData.value?.requested_scopes || 0)),
)

const onAuthorize = async () => {
	try {
		if (!authorizationData.value?.flow_id) {
			throw new Error(formatMessage(messages.noRedirectUrlError))
		}

		const res = await client.labrinth.oauth_internal.accept({
			flow: authorizationData.value.flow_id,
		})

		if (typeof res === 'string') {
			navigateTo(res, {
				external: true,
			})
			return
		}

		throw new Error(formatMessage(messages.noRedirectUrlError))
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: getErrorMessage(err),
			type: 'error',
		})
	}
}

const onReject = async () => {
	try {
		if (!authorizationData.value?.flow_id) {
			throw new Error(formatMessage(messages.noRedirectUrlError))
		}

		const res = await client.labrinth.oauth_internal.reject({
			flow: authorizationData.value.flow_id,
		})

		if (typeof res === 'string') {
			navigateTo(res, {
				external: true,
			})
			return
		}

		throw new Error(formatMessage(messages.noRedirectUrlError))
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: getErrorMessage(err),
			type: 'error',
		})
	}
}

definePageMeta({
	middleware: 'auth',
})
</script>
