// src/config.ts
export const config = {
	siteUrl: import.meta.env.VITE_SITE_URL,
	stripePublishableKey:
		import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY ||
		'pk_test_51JbFxJJygY5LJFfKV50mnXzz3YLvBVe2Gd1jn7ljWAkaBlRz3VQdxN9mXcPSrFbSqxwAb0svte9yhnsmm7qHfcWn00R611Ce7b',
	archonBaseUrl: import.meta.env.VITE_ARCHON_BASE_URL || 'https://archon.modrinth.com',
}
