import type { Component } from 'vue'

export default {
	// Finance
	'payment-statement': () => import('./finance/PaymentStatement.vue'),
} as Record<string, () => Promise<{ default: Component }>>
