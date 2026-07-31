<template>
	<div class="rounded-2xl border border-solid border-surface-4 bg-surface-3">
		<div class="flex flex-col gap-2 border-0 border-b border-solid border-surface-4 p-4">
			<div class="flex flex-wrap justify-between gap-3">
				<div class="flex items-center gap-3">
					<Avatar :src="authorization.app.icon_url" size="48px" />
					<div class="flex flex-col gap-0.5">
						<h2 class="m-0 flex items-center gap-1 text-base font-semibold">
							{{ authorization.app.name }}

							<BadgeCheckIcon
								v-if="isOfficial"
								v-tooltip="formatMessage(messages.officialTooltip)"
								class="size-5 text-green"
								fill="currentColor"
								fill-opacity="0.125"
							/>
						</h2>
						<PageHeaderMetadata>
							<PageHeaderMetadataItem>
								<IntlFormatted :message-id="messages.byLabel">
									<template #~user>
										<nuxt-link
											:to="'/user/' + authorization.owner.id"
											class="flex items-center gap-1 hover:underline"
											target="_blank"
										>
											<Avatar :src="authorization.owner.avatar_url" size="24px" circle />
											{{ authorization.owner.username }}
										</nuxt-link>
									</template>
								</IntlFormatted>
							</PageHeaderMetadataItem>
							<PageHeaderMetadataItem v-if="authorization.app.url">
								<a class="text-link" :to="authorization.app.url" target="_blank">
									{{ authorization.app.url }}
								</a>
							</PageHeaderMetadataItem>
						</PageHeaderMetadata>
					</div>
				</div>
				<div>
					<ButtonStyled color="red">
						<button @click="emit('revoke', authorization.app_id)">
							<XCircleIcon />
							{{ formatMessage(messages.revokeAction) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
			<div v-if="showUnofficialDisclosure" class="flex items-center gap-1 text-orange">
				<IssuesIcon />
				{{ formatMessage(messages.unofficialDisclosure) }}
			</div>
		</div>
		<div class="rounded-b-2xl bg-surface-2 p-4">
			<template v-if="authorization.app.description">
				<label :for="descriptionId" class="mb-1 flex font-semibold text-contrast">
					{{ formatMessage(messages.aboutThisAppLabel) }}
				</label>
				<div :id="descriptionId" class="mb-4">{{ authorization.app.description }}</div>
			</template>

			<label :for="scopeListId" class="mb-1 flex font-semibold text-contrast">
				{{ formatMessage(commonMessages.permissionsLabel) }}
			</label>
			<div :id="scopeListId" class="grid gap-1 sm:grid-cols-2">
				<div
					v-for="scope in scopesToDefinitions(BigInt(authorization.scopes || 0))"
					:key="scope"
					class="flex items-center gap-1"
				>
					<CheckIcon class="size-5 shrink-0 text-green" />
					{{ scope }}
				</div>
			</div>
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { BadgeCheckIcon, CheckIcon, IssuesIcon, XCircleIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	IntlFormatted,
	PageHeaderMetadata,
	PageHeaderMetadataItem,
	useVIntl,
} from '@modrinth/ui'
import { isOfficialAccount } from '@modrinth/utils'

import { useScopes } from '~/composables/auth/scopes.ts'

export type AuthorizationCardData = Labrinth.OAuth.Internal.OAuthClientAuthorization & {
	app: Labrinth.OAuth.Internal.OAuthClient
	owner: Labrinth.Users.v2.User
}

const props = defineProps<{
	authorization: AuthorizationCardData
}>()

const emit = defineEmits<{
	revoke: [appId: string]
}>()

const { formatMessage } = useVIntl()
const { scopesToDefinitions } = useScopes()

const descriptionId = computed(() => `app-description-${props.authorization.id}`)
const scopeListId = computed(() => `app-scope-list-${props.authorization.id}`)

const unofficialTriggerWords = [
	'modrinth',
	'mod rinth',
	'rnodrinth',
	'rinth',
	'modrith',
	'official',
	'verified',
	'verification',
]

const showUnofficialDisclosure = computed(() => {
	return (
		(unofficialTriggerWords.some((word) =>
			props.authorization.app.name.toLowerCase().includes(word),
		) ||
			unofficialTriggerWords.some((word) =>
				props.authorization.owner.username?.toLowerCase().includes(word),
			)) &&
		!isOfficial.value
	)
})

const isOfficial = computed(() => {
	return isOfficialAccount(props.authorization.owner.id)
})

const messages = defineMessages({
	revokeAction: {
		id: 'settings.authorizations.revoke.action',
		defaultMessage: 'Revoke',
	},
	byLabel: {
		id: 'settings.authorizations.created-by',
		defaultMessage: 'Created by {user}',
	},
	aboutThisAppLabel: {
		id: 'settings.authorizations.about-this-app',
		defaultMessage: 'About this app',
	},
	unofficialDisclosure: {
		id: 'settings.authorizations.unofficial-disclosure',
		defaultMessage: 'This app is not affiliated with Modrinth in any way, despite its name.',
	},
	officialTooltip: {
		id: 'settings.authorizations.official-tooltip',
		defaultMessage: 'This app is created by an official Modrinth account.',
	},
})
</script>
