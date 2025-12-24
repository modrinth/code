import IntlMessageFormat from 'intl-messageformat'
import { type CompileError, createI18n, type MessageCompiler, type MessageContext } from 'vue-i18n'

const messageCompiler: MessageCompiler = (msg, { locale, key, onError }) => {
	if (typeof msg === 'string') {
		try {
			const formatter = new IntlMessageFormat(msg, locale)
			return (ctx: MessageContext) => {
				try {
					return formatter.format(ctx.values as Record<string, unknown>) as string
				} catch {
					return String(msg)
				}
			}
		} catch (e) {
			onError?.(e as CompileError)
			return () => key
		}
	} else {
		onError?.(new Error('AST not supported') as CompileError)
		return () => key
	}
}

const i18n = createI18n({
	legacy: false,
	locale: 'en-US',
	fallbackLocale: 'en-US',
	messageCompiler,
	missingWarn: false,
	fallbackWarn: false,
	messages: {
		'en-US': {},
	},
})

export default i18n
