import IntlMessageFormat from 'intl-messageformat'
import type { CompileError, MessageCompiler, MessageContext } from 'vue-i18n'

import enUSMessages from './locales/en-US/index.json'

type CrowdinMessages = Record<string, { message: string } | string>

function transformCrowdinMessages(messages: CrowdinMessages): Record<string, string> {
	const result: Record<string, string> = {}
	for (const [key, value] of Object.entries(messages)) {
		if (typeof value === 'string') {
			result[key] = value
		} else if (typeof value === 'object' && value !== null && 'message' in value) {
			result[key] = value.message
		}
	}
	return result
}

const messageCompiler: MessageCompiler = (msg, { locale, key, onError }) => {
	let messageString: string

	if (typeof msg === 'string') {
		messageString = msg
	} else if (typeof msg === 'object' && msg !== null && 'message' in msg) {
		messageString = (msg as { message: string }).message
	} else {
		onError?.(new Error('Invalid message format') as CompileError)
		return () => key
	}

	try {
		const formatter = new IntlMessageFormat(messageString, locale)
		return (ctx: MessageContext) => {
			try {
				return formatter.format(ctx.values as Record<string, unknown>) as string
			} catch {
				return messageString
			}
		}
	} catch (e) {
		onError?.(e as CompileError)
		return () => key
	}
}

export default defineI18nConfig(() => ({
	legacy: false,
	locale: 'en-US',
	fallbackLocale: 'en-US',
	messageCompiler,
	missingWarn: false,
	fallbackWarn: false,
	messages: {
		'en-US': transformCrowdinMessages(enUSMessages as CrowdinMessages),
	},
}))

export { transformCrowdinMessages }
