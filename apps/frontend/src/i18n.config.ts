import { buildLocaleMessages, createMessageCompiler, type CrowdinMessages } from '@modrinth/ui'

const localeModules = import.meta.glob<{ default: CrowdinMessages }>('./locales/*/index.json', {
	eager: true,
})

export default defineI18nConfig(() => ({
	legacy: false,
	locale: 'en-US',
	fallbackLocale: 'en-US',
	messageCompiler: createMessageCompiler(),
	missingWarn: false,
	fallbackWarn: false,
	messages: buildLocaleMessages(localeModules),
}))
