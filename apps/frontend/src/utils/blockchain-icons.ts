import { PolygonIcon } from '@modrinth/assets'
import type { Component } from 'vue'

export function getBlockchainIcon(blockchain: string): Component | null {
	const lower = blockchain.toLowerCase()

	if (lower.includes('polygon')) return PolygonIcon
	// if (lower.includes('base')) return BaseIcon
	// if (lower.includes('ethereum')) return EthereumIcon
	// if (lower.includes('celo')) return CeloIcon

	return null
}
