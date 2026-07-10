import { defineMessage, useVIntl } from '@modrinth/ui'

import type { Nag, NagContext } from '../../types/nags'

const MAX_LANGUAGE_COUNT = 10
const ALL_LANGUAGE_COUNT = 72

export const serverProjectsNags: Nag[] = [
	{
		id: 'select-country',
		title: defineMessage({
			id: 'nags.select-country.title',
			defaultMessage: 'Select a region',
		}),
		description: defineMessage({
			id: 'nags.select-country.description',
			defaultMessage: 'Let players know what region your server is located in.',
		}),
		status: 'required',
		shouldShow: (context: NagContext) =>
			!!context.projectV3?.minecraft_server && !context.projectV3?.minecraft_server.region,
		link: {
			path: 'settings/server',
			title: defineMessage({
				id: 'nags.server.title',
				defaultMessage: 'Visit server settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-server',
		},
	},
	{
		id: 'too-many-languages',
		title: defineMessage({
			id: 'nags.too-many-languages.title',
			defaultMessage: 'Select accurate languages',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()
			const languageCount = context.projectV3?.minecraft_server?.languages?.length || 0
			const maxLanguageCount = MAX_LANGUAGE_COUNT

			return formatMessage(
				defineMessage({
					id: 'nags.too-many-languages.description',
					defaultMessage:
						"You've selected {languageCount, plural, one {# language} other {# languages}}. Please list only the languages your server actively supports.",
				}),
				{
					languageCount,
					maxLanguageCount,
				},
			)
		},
		status: 'warning',
		shouldShow: (context: NagContext) => {
			const languageCount = context.projectV3?.minecraft_server?.languages?.length || 0
			return (
				languageCount > MAX_LANGUAGE_COUNT &&
				//languageCount <= ALL_LANGUAGE_COUNT &&
				context.projectV3?.minecraft_server != null
			)
		},
		link: {
			path: 'settings/server',
			title: defineMessage({
				id: 'nags.server.title',
				defaultMessage: 'Visit server settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-server',
		},
	},
	{
		id: 'all-languages',
		title: defineMessage({
			id: 'nags.all-languages.title',
			defaultMessage: 'Select accurate languages',
		}),
		description: (context: NagContext) => {
			const { formatMessage } = useVIntl()
			const languageCount = context.projectV3?.minecraft_server?.languages?.length || 0
			const allLanguageCount = ALL_LANGUAGE_COUNT

			return formatMessage(
				defineMessage({
					id: 'nags.all-languages.description',
					defaultMessage:
						"You've selected all available language options. Please list only the languages your server actively supports.",
				}),
				{
					languageCount,
					allLanguageCount,
				},
			)
		},
		status: 'required',
		shouldShow: (context: NagContext) => {
			// const languageCount = context.projectV3?.minecraft_server?.languages?.length || 0
			return false //languageCount >= ALL_LANGUAGE_COUNT && context.projectV3?.minecraft_server != null
		},
		link: {
			path: 'settings/server',
			title: defineMessage({
				id: 'nags.server.title',
				defaultMessage: 'Visit server settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-server',
		},
	},
	{
		id: 'select-language',
		title: defineMessage({
			id: 'nags.select-language.title',
			defaultMessage: 'Select a language',
		}),
		description: defineMessage({
			id: 'nags.select-language.description',
			defaultMessage: 'List the language or languages supported by your server.',
		}),
		status: 'suggestion',
		shouldShow: (context: NagContext) =>
			!!context.projectV3?.minecraft_server &&
			context.projectV3?.minecraft_server?.languages?.length === 0,
		link: {
			path: 'settings/server',
			title: defineMessage({
				id: 'nags.server.title',
				defaultMessage: 'Visit server settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-server',
		},
	},
	{
		id: 'add-java-address',
		title: defineMessage({
			id: 'nags.add-java-address.title',
			defaultMessage: 'Add a Java address',
		}),
		description: defineMessage({
			id: 'nags.add-java-address.description',
			defaultMessage:
				'Add the IP address and port Java Edition players can use to join your server.',
		}),
		status: 'required',
		shouldShow: (context: NagContext) =>
			!!context.projectV3?.minecraft_server && !context.projectV3?.minecraft_java_server?.address,
		link: {
			path: 'settings/server',
			title: defineMessage({
				id: 'nags.server.title',
				defaultMessage: 'Visit server settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-server',
		},
	},
	{
		id: 'select-compatibility',
		title: defineMessage({
			id: 'nags.select-compatibility.title',
			defaultMessage: 'Select compatibility',
		}),
		description: defineMessage({
			id: 'nags.select-compatibility.description',
			defaultMessage:
				'Select what versions your server supports, choose a Modpack, or upload your own.',
		}),
		status: 'required',
		shouldShow: (context: NagContext) => {
			if (
				context.projectV3?.minecraft_java_server?.content?.kind === 'vanilla' &&
				!context.projectV3?.minecraft_java_server?.content?.recommended_game_version
			)
				return true
			return false
		},
		link: {
			path: 'settings/server',
			title: defineMessage({
				id: 'nags.server.title',
				defaultMessage: 'Visit server settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-project-settings-server',
		},
	},
]
