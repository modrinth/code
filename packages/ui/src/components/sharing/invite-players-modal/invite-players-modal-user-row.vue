<template>
	<div
		class="flex h-10 items-center justify-between gap-3 px-6 transition-colors hover:bg-surface-3"
	>
		<div class="flex min-w-0 items-center gap-1.5">
			<AutoLink
				v-tooltip="user.username"
				:to="profileLink"
				:target="profileTarget"
				class="inline-flex min-w-0 items-center gap-1.5"
				:class="profileLink ? 'text-primary hover:underline' : ''"
			>
				<span class="relative flex shrink-0">
					<Avatar
						:src="user.avatarUrl"
						:alt="avatarAlt"
						:tint-by="user.username"
						size="1.5rem"
						circle
						no-shadow
					/>
					<span
						v-if="user.online"
						class="absolute bottom-0 right-0 size-[7px] rounded-full border border-solid border-surface-2 bg-brand"
					/>
				</span>
				<span class="min-w-0 truncate text-base font-medium">
					{{ user.username }}
				</span>
			</AutoLink>
		</div>

		<ButtonStyled v-if="status === 'added'" type="standard" color-fill="none">
			<button disabled>
				<CheckIcon aria-hidden="true" />
				{{ addedLabel }}
			</button>
		</ButtonStyled>
		<ButtonStyled v-else-if="status === 'pending'" type="outlined">
			<button @click="$emit('cancel', user)">
				{{ cancelLabel }}
			</button>
		</ButtonStyled>
		<span v-else-if="status === 'requested'" v-tooltip="requestedTooltip" class="inline-flex">
			<ButtonStyled type="standard" color-fill="none">
				<button disabled>
					{{ requestedLabel }}
				</button>
			</ButtonStyled>
		</span>
		<ButtonStyled v-else color-fill="none">
			<button @click="$emit('invite', user)">
				{{ inviteLabel }}
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon } from '@modrinth/assets'
import { computed } from 'vue'

import AutoLink from '../../base/AutoLink.vue'
import Avatar from '../../base/Avatar.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import type { InvitePlayersUser, InvitePlayersUserProfileLink } from './types'

const props = withDefaults(
	defineProps<{
		user: InvitePlayersUser
		avatarAlt: string
		addedLabel: string
		cancelLabel: string
		inviteLabel: string
		requestedLabel: string
		requestedTooltip: string
		userProfileLink?: (username: string) => InvitePlayersUserProfileLink
	}>(),
	{
		userProfileLink: undefined,
	},
)

defineEmits<{
	invite: [user: InvitePlayersUser]
	cancel: [user: InvitePlayersUser]
}>()

const status = computed(() => props.user.status ?? 'available')
const profileLink = computed(() => getUserProfileLink(props.user.username))
const profileTarget = computed(() =>
	typeof profileLink.value === 'string' && profileLink.value.startsWith('http')
		? '_blank'
		: undefined,
)

function getUserProfileLink(username: string): InvitePlayersUserProfileLink {
	if (!username || username.includes('@')) return undefined
	return props.userProfileLink?.(username) ?? `/user/${encodeURIComponent(username)}`
}
</script>
