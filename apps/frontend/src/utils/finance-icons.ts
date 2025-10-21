import { PolygonIcon, USDCIcon } from '@modrinth/assets';
import type { Component } from 'vue';

export function getCurrencyIcon(currency: string): Component | null {
	const lower = currency.toLocaleLowerCase();
	console.log(lower);

	if (lower.includes('usdc')) return USDCIcon;

	return null;
}

export function getCurrencyColor(currency: string): string {
	const lower = currency.toLowerCase();

	if (lower.includes('usdc')) return 'text-blue'

	return 'text-contrast';
}

export function getBlockchainIcon(blockchain: string): Component | null {
	const lower = blockchain.toLowerCase()

	if (lower.includes('polygon')) return PolygonIcon
	// if (lower.includes('base')) return BaseIcon
	// if (lower.includes('ethereum')) return EthereumIcon
	// if (lower.includes('celo')) return CeloIcon

	return null
}

export function getBlockchainColor(blockchain: string): string {
	const lower = blockchain.toLowerCase();

	if (lower.includes('polygon')) return 'text-blue';
	if (lower.includes('ethereum')) return 'text-purple'

	return 'text-contrast';
}
