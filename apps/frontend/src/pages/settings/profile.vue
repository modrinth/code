<template>
	<section v-if="auth.user" class="universal-card">
		<AccountProfileSettings
			ref="profileSettings"
			:patch-user="patchUser"
			:change-avatar="changeAvatar"
			:delete-avatar="deleteAvatar"
			:get-authenticated-user="getAuthenticatedUser"
			disclaimer-position="bottom"
		/>
		<UnsavedChangesPopup
			:original="profileSettings?.originalState ?? emptyProfileState"
			:modified="profileSettings?.modifiedState ?? emptyProfileState"
			:saving="profileSettings?.saving ?? false"
			@reset="resetProfileSettings"
			@save="saveProfileSettings"
		/>
	</section>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	AccountProfileSettings,
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
const profileSettings = ref<InstanceType<typeof AccountProfileSettings> | null>(null)
const emptyProfileState = {
	username: '',
	bio: '',
	avatarChanged: false,
}

function patchUser(
	userId: string,
	patch: Partial<Pick<Labrinth.Users.v2.User, 'bio' | 'username'>>,
): Promise<void> {
	return client.labrinth.users_v2.patch(userId, patch)
}

function changeAvatar(userId: string, file: Blob, extension: string): Promise<void> {
	return client.labrinth.users_v2.changeIcon(userId, file, extension)
}

function deleteAvatar(userId: string): Promise<void> {
	return client.labrinth.users_v2.deleteIcon(userId)
}

async function getAuthenticatedUser(): Promise<Labrinth.Users.v3.User> {
	const user = await client.labrinth.users_v3.getAuthenticated()
	auth.value.user = user
	return user
}

function resetProfileSettings(): void {
	profileSettings.value?.reset()
}

function saveProfileSettings(): void {
	void profileSettings.value?.save()
}

useHead({
	title: () => `${formatMessage(commonSettingsMessages.profile)} - Modrinth`,
})
</script>
