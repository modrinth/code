import type { Component } from 'vue'

export default {
	// Account
	'auth-method-added': () => import('./templates/account/AuthenticationMethodAdded.vue'),
	'auth-method-removed': () => import('./templates/account/AuthenticationMethodRemoved.vue'),
	'email-changed': () => import('./templates/account/EmailChanged.vue'),
	'password-changed': () => import('./templates/account/PasswordChanged.vue'),
	'password-removed': () => import('./templates/account/PasswordRemoved.vue'),
	'payment-failed': () => import('./templates/account/PaymentFailed.vue'),
	'reset-password': () => import('./templates/account/ResetPassword.vue'),
	'two-factor-added': () => import('./templates/account/TwoFactorAdded.vue'),
	'two-factor-removed': () => import('./templates/account/TwoFactorRemoved.vue'),
	'verify-email': () => import('./templates/account/VerifyEmail.vue'),
	'login-new-device': () => import('./templates/account/LoginNewDevice.vue'),
	'payout-available': () => import('./templates/account/PayoutAvailable.vue'),
	'personal-access-token-created': () => import('./templates/account/PATCreated.vue'),

	// Subscriptions
	'subscription-tax-change': () => import('./templates/account/SubscriptionTaxChange.vue'),

	// Moderation
	'report-submitted': () => import('./templates/moderation/ReportSubmitted.vue'),
	'report-status-updated': () => import('./templates/moderation/ReportStatusUpdated.vue'),
	'moderation-thread-message-received': () =>
		import('./templates/moderation/ModerationThreadMessageReceived.vue'),

	// Project
	'project-status-updated-neutral': () =>
		import('./templates/project/ProjectStatusUpdatedNeutral.vue'),
	'project-status-approved': () => import('./templates/project/ProjectStatusApproved.vue'),
	'project-invited': () => import('./templates/project/ProjectInvited.vue'),
	'project-transferred': () => import('./templates/project/ProjectTransferred.vue'),

	// Organization
	'organization-invited': () => import('./templates/organization/OrganizationInvited.vue'),
} as Record<string, () => Promise<{ default: Component }>>
