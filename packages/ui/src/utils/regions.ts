import { defineMessage, type MessageDescriptor } from '../composables/i18n'

export const regionOverrides = {
	'us-sjc': {
		name: defineMessage({
			id: 'servers.region.north-america-west',
			defaultMessage: 'United States (San Jose)',
		}),
		flag: 'https://flagcdn.com/us.svg',
	},
	'us-dal': {
		name: defineMessage({
			id: 'servers.region.north-america-central',
			defaultMessage: 'United States (Dallas)',
		}),
		flag: 'https://flagcdn.com/us.svg',
	},
	'us-vin': {
		name: defineMessage({
			id: 'servers.region.north-america-east',
			defaultMessage: 'United States (Virginia)',
		}),
		flag: 'https://flagcdn.com/us.svg',
	},
	'eu-cov': {
		name: defineMessage({
			id: 'servers.region.western-europe',
			defaultMessage: 'United Kingdom (Coventry)',
		}),
		flag: 'https://flagcdn.com/gb.svg',
	},
	'eu-lim': {
		name: defineMessage({
			id: 'servers.region.central-europe',
			defaultMessage: 'Germany (Limburg)',
		}),
		flag: 'https://flagcdn.com/de.svg',
	},
	'as-sin': {
		name: defineMessage({
			id: 'servers.region.southeast-asia',
			defaultMessage: 'Singapore',
		}),
		flag: 'https://flagcdn.com/sg.svg',
	},
	'au-syd': {
		name: defineMessage({
			id: 'servers.region.australia',
			defaultMessage: 'Australia (Sydney)',
		}),
		flag: 'https://flagcdn.com/au.svg',
	},
} satisfies Record<string, { name?: MessageDescriptor; flag?: string }>
