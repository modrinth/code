import { defineMessage, type MessageDescriptor } from '@vintl/vintl'

export const regionOverrides = {
  'us-vin': {
    name: defineMessage({ id: 'servers.region.north-america', defaultMessage: 'North America' }),
    flag: 'https://flagcdn.com/us.svg',
  },
  'eu-lim': {
    name: defineMessage({ id: 'servers.region.europe', defaultMessage: 'Europe' }),
    flag: 'https://flagcdn.com/eu.svg',
  },
  'de-fra': {
    name: defineMessage({ id: 'servers.region.europe', defaultMessage: 'Europe' }),
    flag: 'https://flagcdn.com/eu.svg',
  },
} satisfies Record<string, { name?: MessageDescriptor; flag?: string }>
