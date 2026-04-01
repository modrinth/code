<script setup lang="ts">
definePageMeta({
	middleware: ['launcher-auth'],
})

useSeoMeta({
	robots: 'noindex',
})
</script>
<template>
	<div class="px-4 py-6">
		<NuxtPage
			class="auth-surface mx-auto flex max-w-lg flex-col gap-4 border border-solid border-[#c3c3c3] bg-[#EFEFEF] p-4 [&_h1]:m-0 [&_h1]:text-3xl [&_h1]:leading-tight [&_h1]:text-[var(--color-contrast)]"
		/>
	</div>
</template>

<style>
.auth-surface p {
	margin: 0;
}

.auth-surface .btn {
	font-weight: 700;
	min-height: 2.5rem;
	text-decoration: none;
}

.centered-btn {
	margin-inline: auto;
}

.btn.continue-btn svg {
	margin: 0 0 0 0.5rem;
}

.third-party {
	display: grid;
	gap: var(--gap-md);
	grid-template-columns: repeat(2, 1fr);
	width: 100%;
}

.third-party .btn {
	width: 100%;
	vertical-align: middle;
}

.third-party .btn svg {
	margin-right: var(--gap-sm);
	width: 1.25rem;
	height: 1.25rem;
}

@media screen and (max-width: 25.5rem) {
	.third-party .btn {
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
