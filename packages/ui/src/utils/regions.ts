import { defineMessage, type MessageDescriptor } from '@vintl/vintl'

export const regionOverrides = {
  'us-vin': {
    name: defineMessage({ id: 'servers.region.north-america', defaultMessage: 'North America' }),
    flag: 'https://flagcdn.com/us.svg',
  },
  'eu-lim': {
    name: defineMessage({ id: 'servers.region.central-europe', defaultMessage: 'Central Europe' }),
    flag: 'https://flagcdn.com/de.svg',
  },
  'eu-cov': {
    name: defineMessage({ id: 'servers.region.western-europe', defaultMessage: 'Western Europe' }),
    flag: 'https://flagcdn.com/gb.svg',
  },
} satisfies Record<string, { name?: MessageDescriptor; flag?: string }>
