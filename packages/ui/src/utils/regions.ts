import { defineMessage, type MessageDescriptor } from '../composables/i18n'

export const regionOverrides = {
	'us-sjc': {
		name: defineMessage({
			id: 'servers.region.north-america-west',
			defaultMessage: 'Western North America',
		}),
		flag: 'https://flagcdn.com/us.svg',
	},
	'us-dal': {
		name: defineMessage({
			id: 'servers.region.north-america-central',
			defaultMessage: 'Central North America',
		}),
		flag: 'https://flagcdn.com/us.svg',
	},
	'us-vin': {
		name: defineMessage({
			id: 'servers.region.north-america-east',
			defaultMessage: 'Eastern North America',
		}),
		flag: 'https://flagcdn.com/us.svg',
	},
	'eu-cov': {
		name: defineMessage({
			id: 'servers.region.western-europe',
			defaultMessage: 'Western Europe',
		}),
		flag: 'https://flagcdn.com/gb.svg',
	},
	'eu-lim': {
		name: defineMessage({
			id: 'servers.region.central-europe',
			defaultMessage: 'Central Europe',
		}),
		flag: 'https://flagcdn.com/de.svg',
	},
	'as-sin': {
		name: defineMessage({
			id: 'servers.region.southeast-asia',
			defaultMessage: 'Southeast Asia',
		}),
		flag: 'https://flagcdn.com/sg.svg',
	},
	'au-syd': {
		name: defineMessage({
			id: 'servers.region.oceania',
			defaultMessage: 'Oceania',
		}),
		flag: 'https://flagcdn.com/au.svg',
	},
} satisfies Record<string, { name?: MessageDescriptor; flag?: string }>
