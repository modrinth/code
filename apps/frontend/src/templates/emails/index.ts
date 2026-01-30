import type { Component } from 'vue'

export default {
	// Account
	'auth-method-added': () => import('./account/AuthenticationMethodAdded.vue'),
	'auth-method-removed': () => import('./account/AuthenticationMethodRemoved.vue'),
	'email-changed': () => import('./account/EmailChanged.vue'),
	'password-changed': () => import('./account/PasswordChanged.vue'),
	'password-removed': () => import('./account/PasswordRemoved.vue'),
	'payment-failed': () => import('./account/PaymentFailed.vue'),
	'reset-password': () => import('./account/ResetPassword.vue'),
	'two-factor-added': () => import('./account/TwoFactorAdded.vue'),
	'two-factor-removed': () => import('./account/TwoFactorRemoved.vue'),
	'verify-email': () => import('./account/VerifyEmail.vue'),
	'login-new-device': () => import('./account/LoginNewDevice.vue'),
	'payout-available': () => import('./account/PayoutAvailable.vue'),
	'personal-access-token-created': () => import('./account/PATCreated.vue'),

	// Subscriptions
	'subscription-tax-change': () => import('./account/SubscriptionTaxChange.vue'),
	'subscription-credited': () => import('./account/SubscriptionCredited.vue'),

	// Moderation
	'report-submitted': () => import('./moderation/ReportSubmitted.vue'),
	'report-status-updated': () => import('./moderation/ReportStatusUpdated.vue'),
	'moderation-thread-message-received': () =>
		import('./moderation/ModerationThreadMessageReceived.vue'),

	// Project
	'project-status-updated-neutral': () => import('./project/ProjectStatusUpdatedNeutral.vue'),
	'project-status-approved': () => import('./project/ProjectStatusApproved.vue'),
	'project-invited': () => import('./project/ProjectInvited.vue'),
	'project-transferred': () => import('./project/ProjectTransferred.vue'),

	// Organizations
	'organization-invited': () => import('./organization/OrganizationInvited.vue'),
} as Record<string, () => Promise<{ default: Component }>>
