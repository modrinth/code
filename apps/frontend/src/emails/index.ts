import type { Component } from 'vue'

export default {
	'auth-method-added': () => import('./templates/AuthenticationMethodAdded.vue'),
	'auth-method-removed': () => import('./templates/AuthenticationMethodRemoved.vue'),
	'email-changed': () => import('./templates/EmailChanged.vue'),
	'password-changed': () => import('./templates/PasswordChanged.vue'),
	'password-removed': () => import('./templates/PasswordRemoved.vue'),
	'payment-failed': () => import('./templates/PaymentFailed.vue'),
	'reset-password': () => import('./templates/ResetPassword.vue'),
	'two-factor-added': () => import('./templates/TwoFactorAdded.vue'),
	'two-factor-removed': () => import('./templates/TwoFactorRemoved.vue'),
	'verify-email': () => import('./templates/VerifyEmail.vue'),
} as Record<string, () => Promise<Component>>
