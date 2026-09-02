<template>
	<section v-if="auth.user" class="universal-card">
		<AccountSocialSettings
			ref="socialSettings"
			:get-blocked-users="getBlockedUsers"
			:get-users="getUsers"
			:unblock-user="unblockUser"
		/>
		<UnsavedChangesPopup
			:original="socialSettings?.originalState ?? emptySocialState"
			:modified="socialSettings?.modifiedState ?? emptySocialState"
			:saving="socialSettings?.saving ?? false"
			@reset="socialSettings?.reset()"
			@save="socialSettings?.save()"
		/>
	</section>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	AccountSocialSettings,
	commonSettingsMessages,
	injectModrinthClient,
	UnsavedChangesPopup,
	useVIntl,
} from '@modrinth/ui'

definePageMeta({
	middleware: 'auth',
})

const auth = await useAuth()
const client = injectModrinthClient()
const { formatMessage } = useVIntl()
const socialSettings = ref<InstanceType<typeof AccountSocialSettings> | null>(null)
const emptySocialState = {
	friendPrivacy: 'everyone',
	sharedInstancesPrivacy: 'everyone',
	hostingAccessPrivacy: 'everyone',
}

function getBlockedUsers(): Promise<Labrinth.BlockedUsers.v3.BlockedUserId[]> {
	return client.labrinth.blocked_users_v3.list()
}

function getUsers(userIds: string[]): Promise<Labrinth.Users.v2.User[]> {
	return client.labrinth.users_v2.getMultiple(userIds)
}

function unblockUser(userId: string): Promise<void> {
	return client.labrinth.blocked_users_v3.unblock(userId)
}

useHead({
	title: () => `${formatMessage(commonSettingsMessages.social)} - Modrinth`,
})
</script>
