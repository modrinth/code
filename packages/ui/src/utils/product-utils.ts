import type { Labrinth } from '@modrinth/api-client'

export function getProductDisplayName(product: Labrinth.Billing.Internal.Product): string {
	const { metadata } = product

	if (metadata.type === 'pyro') {
		const ramGB = metadata.ram / 1024
		return `${ramGB}GB Server`
	}

	if (metadata.type === 'medal') {
		const ramGB = metadata.ram / 1024
		return `${ramGB}GB Medal Server (${metadata.region})`
	}

	return 'Unknown Product'
}

export function getProductDescription(product: Labrinth.Billing.Internal.Product): string {
	const { metadata } = product

	if (metadata.type === 'pyro') {
		return `${metadata.cpu} vCPU, ${metadata.ram}MB RAM, ${metadata.storage}MB Storage`
	}

	if (metadata.type === 'medal') {
		return `${metadata.cpu} vCPU, ${metadata.ram}MB RAM, ${metadata.storage}MB Storage`
	}

	return ''
}

export function getPriceForInterval(
	product: Labrinth.Billing.Internal.Product,
	currency: string,
	interval: Labrinth.Billing.Internal.PriceDuration,
): number | undefined {
	const productPrice = product.prices.find((x) => x.currency_code === currency)
	if (!productPrice) return undefined

	const { prices } = productPrice

	if (prices.type === 'recurring') {
		return prices.intervals[interval]
	}

	return undefined
}

export const monthsInInterval: Record<'monthly' | 'quarterly' | 'yearly', number> = {
	monthly: 1,
	quarterly: 3,
	yearly: 12,
}
