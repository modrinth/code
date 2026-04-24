<template>
	<div
		class="universal-card flex w-full max-w-[28rem] flex-col gap-6 border border-solid border-surface-5"
	>
		<div v-if="error" class="flex flex-col gap-8">
			<div>
				<h1 class="m-0 mx-auto text-xl font-semibold text-contrast">
					{{ formatMessage(commonMessages.errorLabel) }}
				</h1>
			</div>
			<p class="m-0">
				<span>{{ error.data?.error }}: </span>
				{{ error.data?.description }}
			</p>
		</div>
		<div v-else-if="app && createdBy && authorizationData" class="flex flex-col gap-8">
			<div class="mt-4 flex items-center justify-center">
				<div class="flex w-full flex-row items-center justify-evenly">
					<Avatar size="md" :src="app.icon_url" />
					<!-- <img class="profile-pic" :src="app.icon_url" alt="User profile picture" /> -->
					<div class="flex select-none items-center justify-center text-[2rem] text-primary">→</div>
					<Avatar size="md" circle :src="auth.user.avatar_url" />
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
						<div class="flex flex-row items-center gap-3">
							<div class="flex aspect-square rounded-full bg-green p-2 text-white">
								<CheckIcon />
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

<script setup>
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

import { useScopes } from '@/composables/auth/scopes.ts'

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
	redirectUrl: {
		id: 'auth.authorize.redirect-url',
		defaultMessage: 'You will be redirected to <redirect-url>{url}</redirect-url>',
	},
	title: {
		id: 'auth.authorize.authorize-app-name',
		defaultMessage: 'Authorize {appName}',
	},
})

const router = useNativeRoute()
const auth = await useAuth()
const { scopesToDefinitions } = useScopes()

const clientId = router.query?.client_id || false
const redirectUri = router.query?.redirect_uri || false
const scope = router.query?.scope || false
const state = router.query?.state || false

const getFlowIdAuthorization = async () => {
	const params = {
		client_id: clientId,
		redirect_uri: redirectUri,
		scope,
	}
	if (state) {
		params.state = state
	}

	const authorization = await client.labrinth.oauth_internal.authorize(params)

	if (typeof authorization === 'string') {
		await navigateTo(authorization, {
			external: true,
		})
	}

	return authorization
}

const {
	data: authorizationData,
	isPending: pending,
	error,
	suspense: authSusp,
} = useQuery({
	queryKey: computed(() => ['authorization', clientId, redirectUri, scope, state]),
	queryFn: getFlowIdAuthorization,
	enabled: computed(() => !!clientId && !!redirectUri && !!scope),
})

const { data: app, suspense: appSusp } = useQuery({
	queryKey: computed(() => ['oauth/app', clientId]),
	queryFn: () => client.labrinth.oauth_internal.getApp(clientId),
	enabled: computed(() => !!clientId),
})

const { data: createdBy, suspense: userSusp } = useQuery({
	queryKey: computed(() => ['user', app.value?.created_by]),
	queryFn: () => client.labrinth.users_v2.get(app.value.created_by),
	enabled: computed(() => !!app.value?.created_by),
})

onServerPrefetch(async () => {
	await authSusp()
	await appSusp()
	await userSusp()
})

const scopeDefinitions = computed(() =>
	scopesToDefinitions(BigInt(authorizationData.value?.requested_scopes || 0)),
)

const onAuthorize = async () => {
	try {
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
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

const onReject = async () => {
	try {
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
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

definePageMeta({
	middleware: 'auth',
})
</script>
