import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	addressTitle: { id: 'nags.add-java-address.title', defaultMessage: 'Add a Java address' },
	address: {
		id: 'nags.add-java-address.description',
		defaultMessage: 'Add the IP address and port Java Edition players can use to join your server.',
	},
	languagesTitle: {
		id: 'nags.all-languages.title',
		defaultMessage: 'Select accurate languages',
	},
	tooManyLanguagesTitle: {
		id: 'nags.too-many-languages.title',
		defaultMessage: 'Select accurate languages',
	},
	allLanguages: {
		id: 'nags.all-languages.description',
		defaultMessage: `You've selected all available language options. Please list only the languages your server actively supports.`,
	},
	compatibilityTitle: {
		id: 'nags.select-compatibility.title',
		defaultMessage: 'Select compatibility',
	},
	compatibility: {
		id: 'nags.select-compatibility.description',
		defaultMessage:
			'Select what versions your server supports, choose a Modpack, or upload your own.',
	},
	modpackVersionNotPublicTitle: {
		id: 'nags.modpack-version-not-public.title',
		defaultMessage: 'Select a publicly available modpack version',
	},
	modpackVersionNotPublic: {
		id: 'nags.modpack-version-not-public.description',
		defaultMessage:
			'The selected modpack version is not publicly available. Choose a version that all players can access.',
	},
	countryTitle: { id: 'nags.select-country.title', defaultMessage: 'Select a region' },
	country: {
		id: 'nags.select-country.description',
		defaultMessage: 'Let players know what region your server is located in.',
	},
	languageTitle: { id: 'nags.select-language.title', defaultMessage: 'Select a language' },
	language: {
		id: 'nags.select-language.description',
		defaultMessage: 'List the language or languages supported by your server.',
	},
	tooMany: {
		id: 'nags.too-many-languages.description',
		defaultMessage: `You've selected {languageCount, plural, one {# language} other {# languages}}. Please list only the languages your server actively supports.`,
	},
})

export const serverSettingNags = {
	'add-java-address': {
		title: messages.addressTitle,
		description: messages.address,
		destination: 'server',
	},
	'all-languages': {
		title: messages.languagesTitle,
		description: messages.allLanguages,
		destination: 'server',
	},
	'select-compatibility': {
		title: messages.compatibilityTitle,
		description: messages.compatibility,
		destination: 'server',
	},
	'modpack-version-not-public': {
		title: messages.modpackVersionNotPublicTitle,
		description: messages.modpackVersionNotPublic,
		destination: 'server',
	},
	'select-country': {
		title: messages.countryTitle,
		description: messages.country,
		destination: 'server',
	},
	'select-language': {
		title: messages.languageTitle,
		description: messages.language,
		destination: 'server',
	},
	'too-many-languages': {
		title: messages.tooManyLanguagesTitle,
		description: messages.tooMany,
		destination: 'server',
	},
} satisfies NagDefinitions
