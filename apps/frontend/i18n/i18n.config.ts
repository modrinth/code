import { createMessageCompiler } from '@modrinth/ui'

export default defineI18nConfig(() => ({
	legacy: false,
	locale: 'en-US',
	fallbackLocale: 'en-US',
	messageCompiler: createMessageCompiler(),
	missingWarn: false,
	fallbackWarn: false,
}))
