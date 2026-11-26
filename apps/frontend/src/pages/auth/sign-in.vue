<template>
	<SignInScreen />
</template>

<script setup lang="ts">
import SignInScreen from '@/components/ui/auth/screens/SignInScreen.vue'

useHead({
	meta: [{ name: 'referrer', content: 'no-referrer' }],
})
</script>
