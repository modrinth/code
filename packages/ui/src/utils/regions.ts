import { defineMessage, type MessageDescriptor } from '@vintl/vintl'

export const regionOverrides = {
	'us-vin': {
		name: defineMessage({
			id: 'servers.region.north-america',
			defaultMessage: 'North America',
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
} satisfies Record<string, { name?: MessageDescriptor; flag?: string }>
