import { defineMessages } from '@modrinth/ui'

export const messages = defineMessages({
	serverDetailsHeading: {
		id: 'project.settings.server.details-heading',
		defaultMessage: 'Server details',
	},
	regionLabel: {
		id: 'project.settings.server.region-label',
		defaultMessage: 'Region',
	},
	selectRegionPlaceholder: {
		id: 'project.settings.server.select-region-placeholder',
		defaultMessage: 'Select region',
	},
	languagesLabel: {
		id: 'project.settings.server.languages-label',
		defaultMessage: 'Languages',
	},
	selectLanguagesPlaceholder: {
		id: 'project.settings.server.select-languages-placeholder',
		defaultMessage: 'Select languages',
	},
	javaAddressLabel: {
		id: 'project.settings.server.java-address-label',
		defaultMessage: 'Java address',
	},
	enterAddressPlaceholder: {
		id: 'project.settings.server.enter-address-placeholder',
		defaultMessage: 'Enter address',
	},
	refreshPingTooltip: {
		id: 'project.settings.server.refresh-ping-tooltip',
		defaultMessage: 'Refresh ping',
	},
	serverOnline: {
		id: 'project.settings.server.server-online',
		defaultMessage: 'Server is online!',
	},
	latencyLabel: {
		id: 'project.settings.server.latency-label',
		defaultMessage: 'Latency: {latency}ms',
	},
	pingFailedMessage: {
		id: 'project.settings.server.ping-failed-message',
		defaultMessage:
			"We couldn't ping this server. It may be blocked by your host so try refreshing a few times. If it still doesn't respond please <support-link>contact support</support-link>.",
	},
	srvRecordsHint: {
		id: 'project.settings.server.srv-records-hint',
		defaultMessage:
			"If you have <srv-tooltip>SRV records</srv-tooltip>, you do not need to add a port. Otherwise if you have a port which isn't 25565, you can include it as :12345",
	},
	srvRecordsTooltip: {
		id: 'project.settings.server.srv-records-tooltip',
		defaultMessage:
			'The address you enter here may have DNS SRV records _minecraft._tcp.(your domain) which point to your Minecraft server address and port.',
	},
	bedrockAddressLabel: {
		id: 'project.settings.server.bedrock-address-label',
		defaultMessage: 'Bedrock address',
	},
	cannotSaveTitle: {
		id: 'project.settings.server.cannot-save-title',
		defaultMessage: 'Cannot save',
	},
	cannotSaveText: {
		id: 'project.settings.server.cannot-save-text',
		defaultMessage:
			'The Java server must be reachable before saving. Please ensure the ping succeeds.',
	},
})
