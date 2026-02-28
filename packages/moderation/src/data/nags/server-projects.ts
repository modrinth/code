import { defineMessage } from '@modrinth/ui'
import type { Nag, NagContext } from '../../types/nags'

export const serverProjectsNags: Nag[] = [
	/*
	{
		id: 'add-banner',
		title: defineMessage({
			id: 'nags.add-banner.title',
			defaultMessage: 'Add a banner',
		}),
		description: defineMessage({
			id: 'nags.add-banner.description',
			defaultMessage:
				"Add your server's banner.",
		}),
		status: 'suggestion',
		shouldShow: (context: NagContext) => !!context.projectV3?.minecraft_server,
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.settings.title',
				defaultMessage: 'Visit general settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings',
		},
	},
*/
	{
		id: 'select-country',
		title: defineMessage({
			id: 'nags.select-country.title',
			defaultMessage: 'Select a country',
		}),
		description: defineMessage({
			id: 'nags.select-country.description',
			defaultMessage: 'Let players know what country your server is located in.',
		}),
		status: 'required',
		shouldShow: (context: NagContext) =>
			!!context.projectV3?.minecraft_server && !context.projectV3?.minecraft_server.country,
		link: {
			path: 'settings/server',
			title: defineMessage({
				id: 'nags.server.title',
				defaultMessage: 'Visit server settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-server',
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-server',
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-server',
		},
	},
	{
		id: 'add-bedrock-address',
		title: defineMessage({
			id: 'nags.add-bedrock-address.title',
			defaultMessage: 'Add a Bedrock address',
		}),
		description: defineMessage({
			id: 'nags.add-bedrock-address.description',
			defaultMessage:
				'If your server supports connections from Bedrock Edition players, add the IP address and port they can use to join.',
		}),
		status: 'suggestion',
		shouldShow: (context: NagContext) =>
			!!context.projectV3?.minecraft_server &&
			!context.projectV3?.minecraft_bedrock_server?.address,
		link: {
			path: 'settings/server',
			title: defineMessage({
				id: 'nags.server.title',
				defaultMessage: 'Visit server settings',
			}),
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-server',
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
			shouldShow: (context: NagContext) => context.currentRoute !== 'type-id-settings-server',
		},
	},
]
