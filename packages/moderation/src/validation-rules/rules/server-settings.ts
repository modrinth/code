import { defineMessages } from '@modrinth/ui/i18n'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { toNags } from '../to-nags.ts'
import type { ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	selectCountry: {
		id: 'nags.select-country.title',
		defaultMessage: 'Select a region',
	},
	selectCountryDescription: {
		id: 'nags.select-country.description',
		defaultMessage: 'Let players know what region your server is located in.',
	},
	selectAccurateLanguages: {
		id: 'nags.all-languages.title',
		defaultMessage: 'Select accurate languages',
	},
	allLanguages: {
		id: 'nags.all-languages.description',
		defaultMessage:
			"You've selected all available language options. Please list only the languages your server actively supports.",
	},
	addJavaAddress: {
		id: 'nags.add-java-address.title',
		defaultMessage: 'Add a Java address',
	},
	addJavaAddressDescription: {
		id: 'nags.add-java-address.description',
		defaultMessage: 'Add the IP address and port Java Edition players can use to join your server.',
	},
	selectCompatibility: {
		id: 'nags.select-compatibility.title',
		defaultMessage: 'Select compatibility',
	},
	selectCompatibilityDescription: {
		id: 'nags.select-compatibility.description',
		defaultMessage:
			'Select what versions your server supports, choose a Modpack, or upload your own.',
	},
	tooManyLanguages: {
		id: 'nags.too-many-languages.title',
		defaultMessage: 'Select accurate languages',
	},
	tooManyLanguagesDescription: {
		id: 'nags.too-many-languages.description',
		defaultMessage:
			"You've selected {languageCount, plural, one {# language} other {# languages}}. Please list only the languages your server actively supports.",
	},
	selectLanguage: {
		id: 'nags.select-language.title',
		defaultMessage: 'Select a language',
	},
	selectLanguageDescription: {
		id: 'nags.select-language.description',
		defaultMessage: 'List the language or languages supported by your server.',
	},
})

export const MAX_LANGUAGE_COUNT = 10

export const projectServerSettingsValidationRules = {
	'select-country': {
		severity: 'error',
		evaluate: (context) => ({
			valid:
				!context.projectV3.minecraft_server || Boolean(context.projectV3.minecraft_server.region),
		}),
		presentation: {
			message: messages.selectCountryDescription,
			nag: { title: messages.selectCountry, destination: 'server' },
		},
	},
	'all-languages': {
		severity: 'error',
		evaluate: () => ({ valid: true }),
		presentation: {
			message: messages.allLanguages,
			nag: { title: messages.selectAccurateLanguages, destination: 'server' },
		},
	},
	'add-java-address': {
		severity: 'error',
		evaluate: (context) => ({
			valid:
				!context.projectV3.minecraft_server ||
				Boolean(context.projectV3.minecraft_java_server?.address),
		}),
		presentation: {
			message: messages.addJavaAddressDescription,
			nag: { title: messages.addJavaAddress, destination: 'server' },
		},
	},
	'select-compatibility': {
		severity: 'error',
		evaluate: (context) => ({
			valid:
				context.projectV3.minecraft_java_server?.content?.kind !== 'vanilla' ||
				Boolean(context.projectV3.minecraft_java_server.content.recommended_game_version),
		}),
		presentation: {
			message: messages.selectCompatibilityDescription,
			nag: { title: messages.selectCompatibility, destination: 'server' },
		},
	},
	'too-many-languages': {
		severity: 'warning',
		evaluate: (context) => {
			const languageCount = context.projectV3.minecraft_server?.languages?.length ?? 0
			return languageCount > MAX_LANGUAGE_COUNT
				? { valid: false, values: { languageCount } }
				: { valid: true }
		},
		presentation: {
			message: messages.tooManyLanguagesDescription,
			nag: { title: messages.tooManyLanguages, destination: 'server' },
		},
	},
	'select-language': {
		severity: 'suggestion',
		evaluate: (context) => ({
			valid:
				!context.projectV3.minecraft_server ||
				(context.projectV3.minecraft_server.languages?.length ?? 0) > 0,
		}),
		presentation: {
			message: messages.selectLanguageDescription,
			nag: { title: messages.selectLanguage, destination: 'server' },
		},
	},
} satisfies ValidationRuleSet<ProjectValidationContext>

export function getServerSettingsNags(context: ProjectValidationContext): Nag[] {
	return toNags(evaluateRules(context, projectServerSettingsValidationRules))
}
