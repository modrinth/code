<script setup>
const token = defineModel()
const id = ref(null)

function hCaptchaUpdateToken(newToken) {
	token.value = newToken
}

function hCaptchaReady() {
	window.hCaptchaUpdateToken = hCaptchaUpdateToken
	id.value = window.hcaptcha.render('h-captcha')
}

onMounted(() => {
	if (window.hcaptcha) {
		hCaptchaReady()
	} else {
		window.hCaptchaReady = hCaptchaReady

		useHead({
			script: [
				{
					src: 'https://js.hcaptcha.com/1/api.js?render=explicit&onload=hCaptchaReady',
					async: true,
					defer: true,
				},
			],
		})
	}
})

defineExpose({
	reset: () => {
		token.value = null
		window.hcaptcha.reset(id.value)
	},
})
</script>

<template>
	<div
		id="h-captcha"
		class="h-captcha"
		data-sitekey="4a7a2c80-68f2-4190-9d52-131c76e0c14e"
		:data-theme="$theme.active === 'light' ? 'light' : 'dark'"
		data-callback="hCaptchaUpdateToken"
	></div>
</template>

<style>
.h-captcha {
	display: flex;
	align-items: center;
	justify-content: center;
	overflow: hidden;
	border-radius: var(--radius-md);
	border: 2px solid var(--color-button-bg);
	height: 100px;
	width: 100%;
	max-width: 100%;
}

.h-captcha iframe {
	transform: scale(1.32);
	transform-origin: center;
	margin: -1px;
}

@media screen and (max-width: 400px) {
	.h-captcha {
		height: auto;
	}

	.h-captcha iframe {
		transform: scale(1.03);
		margin: 0;
		max-width: 100%;
	}
}
</style>
