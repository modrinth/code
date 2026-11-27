<template>
	<section v-if="auth.user" class="universal-card">
		<RestrictedActionAlert scope="USER_WRITE" />
		<AccountProfileSettings
			:patch-user="patchUser"
			:change-avatar="changeAvatar"
			:delete-avatar="deleteAvatar"
			:get-authenticated-user="getAuthenticatedUser"
			:prepare-username-change="prepareUsernameChange"
			disclaimer-position="bottom"
		/>
	</section>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	AccountProfileSettings,
	commonSettingsMessages,
	injectModrinthClient,
	useVIntl,
} from '@modrinth/ui'

import RestrictedActionAlert from '~/components/ui/create/RestrictedActionAlert.vue'
import { ensurePrivileged, runPrivileged } from '~/composables/reauth.ts'

definePageMeta({
	middleware: 'auth',
})

const auth = await useAuth()
const client = injectModrinthClient()
const { formatMessage } = useVIntl()

function prepareUsernameChange(): Promise<boolean> {
	return ensurePrivileged()
}

function patchUser(
	userId: string,
	patch: Partial<Pick<Labrinth.Users.v2.User, 'bio' | 'username'>>,
): Promise<void> {
	const request = () => client.labrinth.users_v2.patch(userId, patch)
	if (patch.username != null) {
		return runPrivileged(request)
	}
	return request()
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

useHead({
	title: () => `${formatMessage(commonSettingsMessages.profile)} - Modrinth`,
})
</script>
