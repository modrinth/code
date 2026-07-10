<template>
	<main
		class="relative isolate min-h-screen overflow-hidden bg-surface-1 px-4 pb-16 pt-36 sm:pt-52"
	>
		<div
			class="absolute left-1/2 top-0 -z-10 h-[23.3125rem] w-full -translate-x-1/2 overflow-hidden rounded-none sm:top-10 sm:w-[min(70rem,100%)] sm:rounded-[2.375rem]"
			aria-hidden="true"
		>
			<img
				:src="InviteBackgroundIllustration"
				class="absolute inset-0 size-full scale-[2] object-cover object-[center_25%]"
				alt=""
			/>
			<div
				class="absolute inset-0 bg-[linear-gradient(to_bottom,transparent_20%,var(--surface-1)_100%)]"
			/>
		</div>
		<SharedInstanceInviteOpenInAppModal ref="openInAppModal" />

		<div v-if="isPending" class="relative z-10 flex flex-col items-center gap-4 text-secondary">
			<LoaderCircleIcon class="size-10 animate-spin" />
			<span>{{ formatMessage(messages.loadingInvite) }}</span>
		</div>

		<div
			v-else-if="error || !invite"
			class="relative z-10 mx-auto flex max-w-md flex-col items-center gap-5 text-center"
		>
			<CircleAlertIcon class="size-12 text-secondary" />
			<div class="flex flex-col gap-2">
				<h1 class="m-0 text-2xl font-semibold text-contrast">
					{{ formatMessage(messages.unavailableTitle) }}
				</h1>
				<p class="m-0 text-primary">
					{{ formatMessage(messages.unavailableDescription) }}
				</p>
			</div>
		</div>

		<div
			v-else
			class="relative z-10 mx-auto flex w-full max-w-[26.25rem] flex-col items-center gap-7"
		>
			<div class="invite-avatar relative size-[120px] shrink-0 rounded-full">
				<Avatar
					:src="inviterAvatar"
					:alt="inviterName"
					size="120px"
					circle
					class="!border-0 !shadow-none"
				/>
			</div>

			<div class="flex flex-col items-center gap-4 text-center">
				<div class="flex flex-col items-center gap-2.5">
					<h1 class="m-0 text-2xl font-semibold leading-8 text-contrast">
						{{ formatMessage(messages.invitationTitle, { name: inviterName }) }}
					</h1>
					<div
						class="flex w-max max-w-[calc(100vw-2rem)] flex-nowrap items-center justify-center gap-1.5 overflow-hidden whitespace-nowrap text-lg leading-6 text-primary"
					>
						<span
							class="flex min-w-0 max-w-full items-center gap-1.5 whitespace-nowrap rounded-xl font-semibold text-contrast"
						>
							<Avatar :src="instanceIcon" :alt="invite.instance_name" size="32px" no-shadow />
							<span class="min-w-0 truncate whitespace-nowrap">{{ invite.instance_name }}</span>
						</span>
					</div>
				</div>

				<!-- TODO: Hook this up once the shared instances API exposes invite join counts. -->
				<div
					v-if="false"
					class="items-center gap-2 rounded-full border border-solid border-surface-4 bg-surface-2 px-4 py-2 text-primary"
				>
					{{ formatMessage(messages.joinedCount, { count: 5 }) }}
				</div>
			</div>

			<ButtonStyled color="brand" size="large">
				<button type="button" @click="acceptInvite">
					<UserPlusIcon />
					{{ formatMessage(messages.acceptInvite) }}
				</button>
			</ButtonStyled>

			<div class="flex w-full flex-col gap-2.5">
				<span class="pl-3 font-medium text-primary">{{
					formatMessage(messages.knowThisUser)
				}}</span>
				<Admonition type="neutral" class="shadow-card !text-primary">
					<template #icon="{ iconClass }">
						<CircleAlertIcon :class="[...iconClass, '!text-orange']" />
					</template>
					{{ formatMessage(messages.trustWarning) }}
				</Admonition>
			</div>
		</div>
	</main>
</template>

<script setup lang="ts">
import {
	CircleAlertIcon,
	InviteBackgroundIllustration,
	LoaderCircleIcon,
	UserPlusIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	ButtonStyled,
	defineMessages,
	injectModrinthClient,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, onServerPrefetch, useTemplateRef } from 'vue'

import SharedInstanceInviteOpenInAppModal from '~/components/ui/SharedInstanceInviteOpenInAppModal.vue'

useSeoMeta({
	robots: 'noindex',
})

const { formatMessage } = useVIntl()

const messages = defineMessages({
	loadingInvite: {
		id: 'shared-instance.invite.loading',
		defaultMessage: 'Loading invite…',
	},
	unavailableTitle: {
		id: 'shared-instance.invite.unavailable.title',
		defaultMessage: "This invite isn't available",
	},
	unavailableDescription: {
		id: 'shared-instance.invite.unavailable.description',
		defaultMessage:
			'It may have expired, reached its use limit, or been removed by the person who shared it.',
	},
	invitationTitle: {
		id: 'shared-instance.invite.title',
		defaultMessage: '{name} has invited you to play',
	},
	unknownInviter: {
		id: 'shared-instance.invite.inviter.unknown',
		defaultMessage: 'A Modrinth user',
	},
	joinedCount: {
		id: 'shared-instance.invite.joined-count',
		defaultMessage: '{count, plural, one {# person has} other {# people have}} already joined',
	},
	acceptInvite: {
		id: 'shared-instance.invite.accept',
		defaultMessage: 'Accept invite',
	},
	knowThisUser: {
		id: 'shared-instance.invite.trust.heading',
		defaultMessage: 'Do you know this user?',
	},
	trustWarning: {
		id: 'shared-instance.invite.trust.description',
		defaultMessage:
			'This invite was created by another Modrinth user, not Modrinth. Only accept invites from people you trust.',
	},
})

const route = useRoute()
const client = injectModrinthClient()
const inviteId = computed(() => String(route.params.inviteId))
const openInAppModal = useTemplateRef('openInAppModal')
const instanceIcon = null

const {
	data: invite,
	error,
	isPending,
	suspense,
} = useQuery({
	queryKey: computed(() => ['shared-instance-invite', inviteId.value]),
	queryFn: () => client.sharedinstances.invites_v1.get(inviteId.value),
	retry: false,
})

onServerPrefetch(() => suspense())

const inviter = computed(
	() =>
		invite.value?.managers.find((manager) => manager.type === 'user') ?? invite.value?.managers[0],
)
const inviterName = computed(() => inviter.value?.name ?? formatMessage(messages.unknownInviter))
const inviterAvatar = computed(() => {
	if (!inviter.value) return null
	return inviter.value.type === 'user' ? inviter.value.avatar : inviter.value.icon
})

function acceptInvite() {
	if (!invite.value) return

	openInAppModal.value?.show({
		instance: {
			inviteId: inviteId.value,
			name: invite.value.instance_name,
			inviterName: inviterName.value,
			inviterAvatar: inviterAvatar.value,
			gameVersion: invite.value.game_version,
			loaderVersion: invite.value.loader_version,
		},
	})
}
</script>

<style scoped>
.invite-avatar {
	box-shadow:
		0 0 0 8px rgb(255 255 255 / 5%),
		0 24px 48px rgb(0 0 0 / 3%),
		0 10px 18px rgb(0 0 0 / 3%),
		0 5px 8px rgb(0 0 0 / 4%),
		0 2px 4px rgb(0 0 0 / 4%);
}
</style>
