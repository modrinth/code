<template>
	<div class="main">
		<div class="error-graphic">
			<Logo404 />
		</div>
		<div class="error-box">
			<div class="error-box__body">
				<h1 class="error-box__title">
					{{ formatMessage(messages.title) }}
				</h1>
				<p class="error-box__subtitle">
					{{ formatMessage(messages.subtitle) }}
				</p>
			</div>
		</div>
	</div>
</template>

<script setup>
import { defineMessages, useVIntl } from '@modrinth/ui'

import Logo404 from '~/assets/images/404.svg'

setResponseStatus(404)

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'error.generic.404.title',
		defaultMessage: 'Page not found',
	},
	subtitle: {
		id: 'error.generic.404.subtitle',
		defaultMessage: "The page you were looking for doesn't seem to exist.",
	},
})
</script>

<style lang="scss" scoped>
.main {
	margin: var(--spacing-card-lg) auto;
	width: calc(100% - 4rem);
	min-height: min(90vh, 30rem);
	display: flex;
	flex-direction: column;
	justify-content: center;

	@media screen and (min-width: 800px) {
		width: 600px;
	}
}

.error-graphic {
	margin-bottom: 2rem;
	display: flex;
	justify-content: center;

	svg {
		fill: var(--color-text);
		color: var(--color-text);
		width: min(15rem, 100%);
		height: auto;
	}
}

.error-box {
	background-color: var(--color-raised-bg);
	border-radius: 1.25rem;
	padding: 1.75rem 2rem;
	display: flex;
	flex-direction: column;
	gap: 1.25rem;
	box-shadow: var(--shadow-card);
	position: relative;

	p {
		margin: 0;
	}

	&__title {
		font-size: 1.5rem;
		font-weight: 600;
		margin: 0;
	}

	&__subtitle {
		font-size: 1rem;
		font-weight: 400;
	}

	&__body {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;

		&:empty {
			display: none;
		}
	}
}
</style>
