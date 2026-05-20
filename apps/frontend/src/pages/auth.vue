<script setup lang="ts">
definePageMeta({
	middleware: ['launcher-auth'],
})

useSeoMeta({
	robots: 'noindex',
})
</script>
<template>
	<NuxtPage class="auth-container universal-card" />
</template>

<style>
.auth-container {
	width: 26rem;
	max-width: calc(100% - 2rem);
	margin: 1rem auto;
	display: flex;
	flex-direction: column;
	gap: 2rem;
}

.auth-container h1 {
	font-size: var(--font-size-xl);
	margin: 0 0 -1rem 0;
	color: var(--color-contrast);
}

.auth-container p {
	margin: 0;
}

.auth-container .btn-wrapper :is(a, button) {
	font-weight: 700;
	min-height: 2.5rem;
	text-decoration: none;
}

.centered-btn :is(a, button),
.centered-btn:is(a, button) {
	margin-inline: auto;
}

.continue-btn :is(a, button) svg,
.continue-btn:is(a, button) svg {
	margin: 0 0 0 0.5rem;
}

.third-party {
	display: grid;
	gap: var(--gap-md);
	grid-template-columns: repeat(2, 1fr);
	width: 100%;
}

.third-party a {
	width: 100%;
	vertical-align: middle;
}

.third-party a svg {
	margin-right: var(--gap-sm);
	width: 1.25rem;
	height: 1.25rem;
}

@media screen and (max-width: 25.5rem) {
	.third-party a {
		grid-column: 1 / 3;
	}
}

.auth-form {
	display: flex;
	flex-direction: column;
	gap: var(--gap-md);
}

.auth-form .auth-form__input {
	width: 100%;
	flex-basis: auto;
}

.auth-form__additional-options {
	align-items: center;
	display: flex;
	justify-content: center;
	gap: var(--gap-md);
	flex-wrap: wrap;
}

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
