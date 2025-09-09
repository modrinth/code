import AuthenticationMethodAdded from './templates/AuthenticationMethodAdded.vue'
import AuthenticationMethodRemoved from './templates/AuthenticationMethodRemoved.vue'
import EmailChanged from './templates/EmailChanged.vue'
import PasswordChanged from './templates/PasswordChanged.vue'
import PasswordRemoved from './templates/PasswordRemoved.vue'
import PaymentFailed from './templates/PaymentFailed.vue'
import ResetPassword from './templates/ResetPassword.vue'
import TwoFactorAdded from './templates/TwoFactorAdded.vue'
import TwoFactorRemoved from './templates/TwoFactorRemoved.vue'
import VerifyEmail from './templates/VerifyEmail.vue'

export default {
	'auth-method-added': AuthenticationMethodAdded,
	'auth-method-removed': AuthenticationMethodRemoved,
	'email-changed': EmailChanged,
	'password-changed': PasswordChanged,
	'password-removed': PasswordRemoved,
	'payment-failed': PaymentFailed,
	'reset-password': ResetPassword,
	'two-factor-added': TwoFactorAdded,
	'two-factor-removed': TwoFactorRemoved,
	'verify-email': VerifyEmail,
} as Record<string, Component>
