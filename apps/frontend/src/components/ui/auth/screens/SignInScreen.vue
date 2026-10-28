<template>
	<SignInView v-if="showSignIn" />
</template>

<script setup lang="ts">
import SignInView from '@/components/ui/auth/SignIn.vue'
import { provideSignIn } from '@/composables/sign-in.ts'

const { showSignIn, loadAuth, redirectOAuthState, finishSessionCode } = provideSignIn()
await loadAuth()
await redirectOAuthState()
await finishSessionCode()
</script>
