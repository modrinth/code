<template>
	<div class="flex w-full flex-col rounded-2xl border border-solid border-surface-5 bg-surface-4">
		<button
			v-for="account in accounts"
			:key="account.id"
			type="button"
			data-button
			:class="rowClass"
			@click="emit('select', account)"
		>
			<Avatar :src="account.avatarUrl" size="32px" circle />
			<span class="min-w-0 truncate font-medium text-contrast">{{ account.username }}</span>
			<UserRoleIcon :role="account.role" class="!size-5" />
			<RightArrowIcon aria-hidden="true" class="ml-auto h-5 w-5 shrink-0 text-secondary" />
		</button>
		<button v-if="addAccountLabel" type="button" data-button :class="rowClass" @click="emit('add')">
			<span
				class="flex size-8 shrink-0 items-center justify-center rounded-full bg-surface-5 text-secondary"
			>
				<PlusIcon aria-hidden="true" class="h-5 w-5" />
			</span>
			<span class="font-medium text-contrast">{{ addAccountLabel }}</span>
			<RightArrowIcon aria-hidden="true" class="ml-auto h-5 w-5 shrink-0 text-secondary" />
		</button>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { PlusIcon, RightArrowIcon } from '@modrinth/assets'

import Avatar from '../base/Avatar.vue'
import UserRoleIcon from './UserRoleIcon.vue'

export type AccountChoice = {
	id: string
	username: string
	avatarUrl?: string | null
	role?: Labrinth.Users.v2.Role | null
}

defineProps<{
	accounts: AccountChoice[]
	addAccountLabel?: string
}>()

const emit = defineEmits<{
	select: [account: AccountChoice]
	add: []
}>()

const rowClass =
	'flex w-full !w-full items-center gap-2 border-0 border-t border-solid border-surface-5 bg-surface-4 px-4 py-2 text-left transition-colors first:rounded-t-2xl first:border-t-0 last:rounded-b-2xl hover:bg-surface-5'
</script>
