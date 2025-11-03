import { PolygonIcon, USDCColorIcon } from '@modrinth/assets'
import type { Component } from 'vue'

export function getCurrencyIcon(currency: string): Component | null {
	const lower = currency.toLocaleLowerCase()

	if (lower.includes('usdc')) return USDCColorIcon

	return null
}

export function getCurrencyColor(currency: string): string {
	const lower = currency.toLowerCase()

	if (lower.includes('usdc')) return 'text-blue'

	return 'text-contrast'
}

export function getBlockchainIcon(blockchain: string): Component | null {
	const lower = blockchain.toLowerCase()

	if (lower.includes('polygon')) return PolygonIcon

	return null
}

export function getBlockchainColor(blockchain: string): string {
	const lower = blockchain.toLowerCase()

	if (lower.includes('polygon')) return 'text-purple'

	return 'text-contrast'
}
