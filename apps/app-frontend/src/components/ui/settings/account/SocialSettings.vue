<template>
	<AccountSocialSettings
		ref="socialSettings"
		:get-blocked-users="get_blocked_users"
		:get-users="getUsers"
		:unblock-user="unblock_user"
	/>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { AccountSocialSettings, injectModrinthClient } from '@modrinth/ui'
import { inject, onBeforeUnmount, onMounted, ref } from 'vue'

import { get_blocked_users, unblock_user } from '@/helpers/users'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

const client = injectModrinthClient()
const settingsModal = inject(appSettingsModalContextKey, null)
const socialSettings = ref<InstanceType<typeof AccountSocialSettings> | null>(null)

onMounted(() => {
	settingsModal?.registerUnsavedChangesController({
		hasChanges: () => socialSettings.value?.hasChanges ?? false,
		getOriginal: () => socialSettings.value?.originalState ?? {},
		getModified: () => socialSettings.value?.modifiedState ?? {},
		isSaving: () => socialSettings.value?.saving ?? false,
		reset: () => socialSettings.value?.reset(),
		save: () => socialSettings.value?.save(),
	})
})

onBeforeUnmount(() => {
	settingsModal?.registerUnsavedChangesController(null)
})

function getUsers(userIds: string[]): Promise<Labrinth.Users.v2.User[]> {
	return client.labrinth.users_v2.getMultiple(userIds)
}
</script>
