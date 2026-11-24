import { computed, ref } from 'vue'

import { defineMessage, defineMessages, useVIntl } from '../../../composables/i18n'

export const urlLabel = defineMessage({
	id: 'markdown-editor.url-label',
	defaultMessage: 'URL',
})

const messages = defineMessages({
	urlValidationErrorMalformed: {
		id: 'markdown-editor.url-validation-error.malformed',
		defaultMessage: 'Invalid URL. Make sure the URL is well-formed.',
	},
	urlValidationErrorUnsupportedProtocol: {
		id: 'markdown-editor.url-validation-error.unsupported-protocol',
		defaultMessage: 'Unsupported protocol. Use http or https.',
	},
	urlValidationErrorBlockedDomain: {
		id: 'markdown-editor.url-validation-error.blocked-domain',
		defaultMessage: 'Invalid URL. This domain is not allowed.',
	},
})

export function parseMarkdownUrl(input: string) {
	if (!input) return {}

	let url: URL
	try {
		url = new URL(input)
	} catch {
		return { error: messages.urlValidationErrorMalformed }
	}
	if (url.protocol !== 'http:' && url.protocol !== 'https:') {
		return { error: messages.urlValidationErrorUnsupportedProtocol }
	}
	if (
		['forgecdn', 'cdn.discordapp', 'media.discordapp'].some((domain) =>
			url.hostname.includes(domain),
		)
	) {
		return { error: messages.urlValidationErrorBlockedDomain }
	}
	if (url.protocol === 'http:') url.protocol = 'https:'

	return { url: url.toString() }
}

export function useMarkdownUrl() {
	const { formatMessage } = useVIntl()
	const input = ref('')
	const parsed = computed(() => parseMarkdownUrl(input.value))
	const url = computed(() => parsed.value.url)
	const error = computed(() => parsed.value.error && formatMessage(parsed.value.error))

	return { input, url, error }
}
