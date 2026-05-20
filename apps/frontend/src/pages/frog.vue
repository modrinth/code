<script setup lang="ts">
import { defineMessages, useVIntl } from '@modrinth/ui'

const vintl = useVIntl()
const { formatMessage } = vintl

const messages = defineMessages({
	frogTitle: {
		id: 'frog.title',
		defaultMessage: 'Frog',
	},
	frogDescription: {
		id: 'frog',
		defaultMessage: "You've been frogged! 🐸",
	},
	frogAltText: {
		id: 'frog.altText',
		defaultMessage: 'A photorealistic painting of a frog labyrinth',
	},
})

useSeoMeta({
	robots: 'noindex',
})
</script>

<template>
	<div class="card">
		<h1>{{ formatMessage(messages.frogTitle) }}</h1>
		<p>{{ formatMessage(messages.frogDescription) }}</p>
		<img src="https://cdn.modrinth.com/frog.png" :alt="formatMessage(messages.frogAltText)" />
	</div>
</template>

<style lang="scss" scoped>
.card {
	width: calc(100% - 2 * var(--spacing-card-md));
	max-width: 1280px;
	margin-inline: auto;
	text-align: center;
	box-sizing: border-box;
	margin-block: var(--spacing-card-md);
}

img {
	margin-block: 0 1.5rem;
	width: 60%;
	max-width: 40rem;
}
</style>
