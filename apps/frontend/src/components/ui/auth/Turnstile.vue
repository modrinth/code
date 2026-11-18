<script setup>
const TURNSTILE_SITE_KEY = '0x4AAAAAAFbxS3P9EOjXZD1_'

const token = defineModel()
const container = ref(null)
const id = ref(null)
const theme = useTheme()

function turnstileReady() {
	id.value = window.turnstile.render(container.value, {
		sitekey: TURNSTILE_SITE_KEY,
		theme: theme.active === 'light' ? 'light' : 'dark',
		size: 'flexible',
		callback: (newToken) => {
			token.value = newToken
		},
		'expired-callback': () => {
			token.value = null
		},
	})
}

onMounted(() => {
	if (window.turnstile) {
		turnstileReady()
	} else {
		window.turnstileReady = turnstileReady

		useHead({
			script: [
				{
					src: 'https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit&onload=turnstileReady',
					async: true,
					defer: true,
				},
			],
		})
	}
})

onBeforeUnmount(() => {
	if (id.value !== null) window.turnstile.remove(id.value)
})

defineExpose({
	reset: () => {
		token.value = null
		window.turnstile.reset(id.value)
	},
})
</script>

<template>
	<div ref="container" class="turnstile"></div>
</template>

<style>
.turnstile {
	display: flex;
	justify-content: center;
	overflow: hidden;
	border-radius: var(--radius-md);
	border: 2px solid var(--color-button-bg);
	height: 65px;
	width: 100%;

	> div {
		position: relative;
		top: -2px;
		min-width: calc(100% + 4px);
	}
}
</style>
