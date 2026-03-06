import {
	buildLocaleMessages,
	createMessageCompiler,
	type CrowdinMessages,
	uiLocaleModules,
} from '@modrinth/ui'
import { createI18n } from 'vue-i18n'

const localeModules = import.meta.glob<{ default: CrowdinMessages }>('./locales/*/index.json', {
	eager: true,
})

async function resolveModules(
	modules: Record<string, () => Promise<{ default: CrowdinMessages }>>,
): Promise<Record<string, { default: CrowdinMessages }>> {
	return Object.fromEntries(
		await Promise.all(Object.entries(modules).map(async ([path, load]) => [path, await load()])),
	)
}

const i18n = createI18n({
	legacy: false,
	locale: 'en-US',
	fallbackLocale: 'en-US',
	messageCompiler: createMessageCompiler(),
	missingWarn: false,
	fallbackWarn: false,
	messages: buildLocaleMessages(localeModules, await resolveModules(uiLocaleModules)),
})

export default i18n
