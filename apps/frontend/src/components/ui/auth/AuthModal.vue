<template>
	<NewModal
		ref="modal"
		hide-header
		merge-header
		no-padding
		scrollable
		width="27rem"
		max-width="calc(100vw - 2rem)"
		max-content-height="min(40rem, calc(100vh - 4rem))"
		:before-hide="allowHide"
		:on-hide="onHide"
	>
		<Suspense>
			<component :is="screen" v-if="screen && location" :key="location.fullPath" />
			<template #fallback>
				<div class="min-h-40" />
			</template>
		</Suspense>
	</NewModal>
</template>

<script setup lang="ts">
import { NewModal } from '@modrinth/ui'
import { type Component, computed, nextTick, onUnmounted, provide, ref, watch } from 'vue'

import ReauthenticateScreen from '@/components/ui/auth/screens/ReauthenticateScreen.vue'
import SignInScreen from '@/components/ui/auth/screens/SignInScreen.vue'
import SignUpScreen from '@/components/ui/auth/screens/SignUpScreen.vue'
import VerifyEmailScreen from '@/components/ui/auth/screens/VerifyEmailScreen.vue'
import {
	AUTH_ROUTE_KEY,
	authModalLocation,
	authModalReady,
	type EmbeddableAuthPath,
} from '@/composables/auth-modal.ts'
import { authProviderHandoff } from '@/composables/external-auth.ts'

const screens: Record<EmbeddableAuthPath, Component> = {
	'/auth/sign-in': SignInScreen,
	'/auth/sign-up': SignUpScreen,
	'/auth/reauthenticate': ReauthenticateScreen,
	'/auth/verify-email': VerifyEmailScreen,
}

const modal = ref<InstanceType<typeof NewModal>>()
const showing = ref(false)
const route = useRoute()
const location = computed(() => authModalLocation.value)

provide(AUTH_ROUTE_KEY, location)
authModalReady.value = true
onUnmounted(() => {
	authModalReady.value = false
})

const screen = computed(() => {
	const path = location.value?.path
	if (!path || !(path in screens)) {
		return null
	}
	return screens[path as EmbeddableAuthPath]
})

watch(
	location,
	async (next) => {
		if (next) {
			if (showing.value) {
				return
			}
			showing.value = true
			await nextTick()
			modal.value?.show()
			return
		}
		if (!showing.value) {
			return
		}
		showing.value = false
		modal.value?.hide()
	},
	{ immediate: true },
)

watch(
	() => route.fullPath,
	() => {
		if (!authModalLocation.value || authProviderHandoff.value) {
			return
		}
		authModalLocation.value = null
	},
)

function allowHide() {
	return !authProviderHandoff.value
}

function onHide() {
	if (!authModalLocation.value || authProviderHandoff.value) {
		return
	}
	showing.value = false
	authModalLocation.value = null
}
</script>
