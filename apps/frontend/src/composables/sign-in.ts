import {
	commonMessages,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import type { ComputedRef, InjectionKey, Ref } from 'vue'
import type { LocationQuery } from 'vue-router'

import {
	hydrateStoredAccounts,
	isStoredAccountAuthMethod,
	readPendingAuthMethod,
	rememberStoredAccount,
	setPendingAuthMethod,
	type StoredAccount,
	type StoredAccountAuthMethod,
	updateStoredAccountAuthMethod,
	useLastSignInAuthMethod,
	useStoredAccounts,
} from '@/composables/accounts.ts'
import {
	ADD_ACCOUNT_QUERY_PARAM,
	promotePendingSignInOAuthProvider,
	useAuthState,
} from '@/composables/auth.ts'
import { getAuthErrorMessage } from '@/composables/auth-form.ts'
import { finishAuth, useAuthPageTitle, useAuthRoute } from '@/composables/auth-modal.ts'
import {
	createLauncherHandoff,
	getQueryString,
	hideLauncherSessionCode,
	isLauncherProtocolV2,
	LAUNCHER_APP_CODE_QUERY_PARAM,
	LAUNCHER_REAUTH_ACCOUNT_STORAGE_KEY,
	launcherAuthMessages,
	type LauncherHandoff,
} from '@/composables/launcher-auth.ts'
import { getPasskeyCredential } from '@/helpers/passkey.ts'

interface AuthGlobalsResponse {
	captcha_enabled?: boolean
	[key: string]: unknown
}

type LauncherCallback = Extract<LauncherHandoff, { type: 'callback' }>

interface ReauthAccountPreview {
	id: string
	username: string
	avatarUrl?: string | null
	authMethod: StoredAccountAuthMethod | null
}

interface SignInContext {
	showSignIn: ComputedRef<boolean>
	launcherHandoff: Ref<LauncherCallback | null>
	flow: Ref<string>
	twoFactorCode: Ref<string>
	twoFactorPending: Ref<boolean>
	twoFactorError: Ref<boolean>
	accounts: ComputedRef<StoredAccount[]>
	reauthAccount: ComputedRef<ReauthAccountPreview | null>
	redirectTarget: string
	routeQuery: ComputedRef<LocationQuery>
	globals: Ref<AuthGlobalsResponse | undefined>
	email: Ref<string>
	password: Ref<string>
	token: Ref<string>
	beginPasswordSignIn: () => Promise<void>
	begin2FASignIn: (code: string) => Promise<void>
	beginPasskeySignin: () => Promise<void>
	setCaptchaRef: (captchaRef: unknown) => void
	cancelReauth: () => void
	onSelectLauncherAccount: (account: { id: string }) => void
}

const SIGN_IN_KEY: InjectionKey<SignInContext> = Symbol('sign-in')

export function provideSignIn() {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { addNotification } = injectNotificationManager()
	const { formatMessage } = useVIntl()

	const messages = defineMessages({
		signInTitle: {
			id: 'auth.sign-in.title',
			defaultMessage: 'Sign In',
		},
	})

	useAuthPageTitle(() => formatMessage(messages.signInTitle))

	const route = useAuthRoute()
	const auth = useAuthState()
	const lastSignInOAuthProvider = useLastSignInAuthMethod()

	const redirectTarget = getQueryString(route.query.redirect) ?? ''
	const routeQuery = computed(() => route.query)
	const launcherHandoff = ref<LauncherCallback | null>(null)
	const reauthAccount = ref<StoredAccount | null>(null)
	const isProtocolV2 = isLauncherProtocolV2(route)

	const isAddingAccount = route.query[ADD_ACCOUNT_QUERY_PARAM] !== undefined
	const isLauncherSignIn = route.query.launcher !== undefined
	const storedAccounts = useStoredAccounts()
	const signInReady = ref(!isLauncherSignIn)
	const email = ref('')
	const password = ref('')
	const token = ref('')

	function isLauncherSignInMethod(value: unknown): value is StoredAccountAuthMethod {
		return isStoredAccountAuthMethod(value) && value !== 'paypal'
	}

	function readLastSignInMethod(): StoredAccountAuthMethod | undefined {
		return isLauncherSignInMethod(lastSignInOAuthProvider.value)
			? lastSignInOAuthProvider.value
			: undefined
	}

	function inferLauncherAuthMethod(
		account: StoredAccount,
		accountCount: number,
	): StoredAccountAuthMethod | undefined {
		if (isLauncherSignInMethod(account.authMethod)) {
			return account.authMethod
		}

		const user = auth.value.user
		const isCurrentUser = user?.id === account.id
		const lastMethod = readLastSignInMethod()
		if ((isCurrentUser || accountCount === 1) && lastMethod) {
			return lastMethod
		}
		if (!isCurrentUser || !user) {
			return undefined
		}

		const linked = (user.auth_providers ?? []).filter(isLauncherSignInMethod)
		if (linked.length === 1) {
			return linked[0]
		}
		if (linked.length === 0 && user.has_password) {
			return 'password'
		}
		return undefined
	}

	const choosableAccounts = computed((): StoredAccount[] => {
		const user = auth.value.user
		const token = auth.value.token
		const accounts = storedAccounts.value.map((stored) => {
			if (user && token && stored.id === user.id) {
				return {
					...stored,
					username: user.username,
					avatarUrl: user.avatar_url ?? stored.avatarUrl,
					token,
					role: user.role,
				}
			}

			return stored
		})

		if (user && token && !accounts.some((account) => account.id === user.id)) {
			accounts.push({
				id: user.id,
				username: user.username,
				avatarUrl: user.avatar_url ?? null,
				token,
				role: user.role,
			})
		}

		return accounts.map((account) => ({
			...account,
			authMethod: inferLauncherAuthMethod(account, accounts.length),
		}))
	})

	const launcherAccountChoices = computed(() => {
		if (!isLauncherSignIn) {
			return []
		}

		const minimumAccounts = isProtocolV2 || isAddingAccount ? 1 : 2
		return choosableAccounts.value.length >= minimumAccounts ? choosableAccounts.value : []
	})

	const accounts = computed(() => (reauthAccount.value ? [] : launcherAccountChoices.value))

	const reauthAccountPreview = computed(() => {
		const account = reauthAccount.value
		if (!account) {
			return null
		}
		return {
			id: account.id,
			username: account.username,
			avatarUrl: account.avatarUrl,
			authMethod: account.authMethod ?? null,
		}
	})

	function sessionTokensFromRoute() {
		const session = getQueryString(route.query.code) ?? ''
		const appSession = getQueryString(route.query[LAUNCHER_APP_CODE_QUERY_PARAM]) ?? ''
		return {
			session: session.startsWith('mra_') ? session : null,
			appSession: appSession.startsWith('mra_') ? appSession : null,
		}
	}

	onMounted(async () => {
		if (!isLauncherSignIn) {
			return
		}

		hydrateStoredAccounts()

		if (isProtocolV2) {
			const { session, appSession } = sessionTokensFromRoute()
			if (session) {
				await completeProtocolV2SignIn(session, appSession, readPendingAuthMethod())
			}
		}

		if (launcherHandoff.value) {
			signInReady.value = true
			return
		}

		if (
			!isProtocolV2 &&
			auth.value.user &&
			!isAddingAccount &&
			choosableAccounts.value.length === 1 &&
			route.query.code === undefined
		) {
			await showLauncherOpeningPage(auth.value.token)
			if (launcherHandoff.value) {
				signInReady.value = true
			}
			return
		}

		signInReady.value = true
	})

	async function showLauncherOpeningPage(sessionToken: string) {
		try {
			const handoff = await createLauncherHandoff(route, sessionToken)
			if (handoff.type === 'external') {
				promotePendingSignInOAuthProvider()
				await navigateTo(handoff.url, {
					external: true,
				})
				return
			}

			promotePendingSignInOAuthProvider()
			launcherHandoff.value = handoff
			if (isProtocolV2) {
				hideLauncherSessionCode()
			}
		} catch (err) {
			console.error(err)
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: formatMessage(launcherAuthMessages.handoffFailed),
				type: 'error',
			})
		}
	}

	function onSelectLauncherAccount(account: { id: string }) {
		const stored = choosableAccounts.value.find((choice) => choice.id === account.id)
		if (!stored) {
			return
		}

		if (!isProtocolV2) {
			void showLauncherOpeningPage(stored.token)
			return
		}

		reauthAccount.value = stored
		email.value = stored.username
		password.value = ''
		if (import.meta.client) {
			window.sessionStorage.setItem(LAUNCHER_REAUTH_ACCOUNT_STORAGE_KEY, stored.id)
		}

		const original = storedAccounts.value.find((choice) => choice.id === stored.id)
		if (stored.authMethod && stored.authMethod !== original?.authMethod) {
			updateStoredAccountAuthMethod(stored, stored.authMethod)
		}
	}

	function cancelReauth() {
		reauthAccount.value = null
		email.value = ''
		password.value = ''
		if (import.meta.client) {
			window.sessionStorage.removeItem(LAUNCHER_REAUTH_ACCOUNT_STORAGE_KEY)
		}
	}

	function rememberLauncherAuthMethod(authMethod?: StoredAccountAuthMethod) {
		if (!authMethod || !import.meta.client) {
			return
		}

		const accountId = window.sessionStorage.getItem(LAUNCHER_REAUTH_ACCOUNT_STORAGE_KEY)
		const account =
			(accountId ? choosableAccounts.value.find((stored) => stored.id === accountId) : undefined) ??
			reauthAccount.value
		if (!account) {
			return
		}

		updateStoredAccountAuthMethod(account, authMethod)
		window.sessionStorage.removeItem(LAUNCHER_REAUTH_ACCOUNT_STORAGE_KEY)
	}

	const captcha = ref<{ reset?: () => void } | null>(null)
	const setCaptchaRef = (captchaRef: unknown) => {
		captcha.value = (captchaRef as { reset?: () => void } | null) ?? null
	}

	const { data: globals } = useQuery<AuthGlobalsResponse>({
		queryKey: ['auth-globals'],
		queryFn: async () => {
			try {
				return await client.labrinth.globals_internal.get()
			} catch (err) {
				console.error('Error fetching globals:', err)
				return { captcha_enabled: true, tax_compliance_thresholds: {} }
			}
		},
	})

	const flow = ref(getQueryString(route.query.flow) ?? '')

	async function beginPasswordSignIn() {
		setPendingAuthMethod(null)
		lastSignInOAuthProvider.value = null
		startLoading()
		try {
			const res = await client.labrinth.auth_v2.login({
				username: email.value,
				password: password.value,
				challenge: token.value,
				app_session: shouldRequestAppSession(),
			})

			if (res.flow) {
				flow.value = res.flow
			} else {
				await finishSignIn(res.session, res.app_session, 'password')
			}
		} catch (err) {
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: getAuthErrorMessage(err),
				type: 'error',
			})
			captcha.value?.reset?.()
		}
		stopLoading()
	}

	const twoFactorCode = ref('')
	const twoFactorPending = ref(false)
	const twoFactorError = ref(false)

	async function begin2FASignIn(code: string) {
		if (twoFactorPending.value) {
			return
		}
		twoFactorPending.value = true
		twoFactorError.value = false
		startLoading()
		try {
			const res = await client.labrinth.auth_v2.login2FA({
				flow: flow.value,
				code,
				app_session: shouldRequestAppSession(),
			})

			await finishSignIn(res.session, res.app_session, 'password')
		} catch {
			twoFactorCode.value = ''
			twoFactorError.value = true
		} finally {
			twoFactorPending.value = false
			stopLoading()
		}
	}

	async function beginPasskeySignin() {
		startLoading()
		try {
			const start = await client.labrinth.auth_v2.authenticatePasskeyStart()

			const credential = await getPasskeyCredential(start.options.publicKey)

			const result = await client.labrinth.auth_v2.authenticatePasskeyFinish({
				flow: start.flow,
				credential,
				app_session: shouldRequestAppSession(),
			})

			setPendingAuthMethod('passkey')
			await finishSignIn(result.session, result.app_session, 'passkey')
		} catch (err) {
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: getAuthErrorMessage(err),
				type: 'error',
			})
		}
		stopLoading()
	}

	function isKnownAccountReauth() {
		if (reauthAccount.value) {
			return true
		}
		if (!import.meta.client) {
			return false
		}
		return window.sessionStorage.getItem(LAUNCHER_REAUTH_ACCOUNT_STORAGE_KEY) != null
	}

	const shouldRequestAppSession = () => isProtocolV2 && !isKnownAccountReauth()

	async function adoptWebsiteSession(sessionToken: string, authMethod?: StoredAccountAuthMethod) {
		await useAuth(sessionToken)
		await useUser()
		queryClient.clear()

		const signedIn = await useAuth()
		if (signedIn.value.user && signedIn.value.token) {
			rememberStoredAccount(
				signedIn.value.user,
				signedIn.value.token,
				authMethod ? { authMethod } : undefined,
			)
		}
	}

	async function completeProtocolV2SignIn(
		sessionToken: string,
		appSessionToken: string | null | undefined,
		authMethod?: StoredAccountAuthMethod,
	) {
		const knownAccount = isKnownAccountReauth()

		if (!knownAccount) {
			if (!appSessionToken) {
				addNotification({
					title: formatMessage(commonMessages.errorNotificationTitle),
					text: formatMessage(launcherAuthMessages.handoffFailed),
					type: 'error',
				})
				return
			}

			try {
				await adoptWebsiteSession(sessionToken, authMethod)
			} catch (err) {
				addNotification({
					title: formatMessage(commonMessages.errorNotificationTitle),
					text: getAuthErrorMessage(err),
					type: 'error',
				})
				return
			}
		} else {
			rememberLauncherAuthMethod(authMethod)
		}

		await showLauncherOpeningPage(appSessionToken ?? sessionToken)
	}

	async function finishSignIn(
		sessionToken?: string | null,
		appSessionToken?: string | null,
		authMethod?: StoredAccountAuthMethod,
	) {
		if (route.query.launcher) {
			if (isProtocolV2) {
				if (sessionToken) {
					await completeProtocolV2SignIn(sessionToken, appSessionToken, authMethod)
				}
				return
			}

			const sessionTokenForLauncher = sessionToken ?? auth.value.token
			if (sessionTokenForLauncher) {
				await showLauncherOpeningPage(sessionTokenForLauncher)
			}

			return
		}

		if (sessionToken) {
			await useAuth(sessionToken)
			await useUser()
			queryClient.clear()
		}

		const signedIn = await useAuth()
		if (signedIn.value.user && signedIn.value.token) {
			const nextAuthMethod = authMethod ?? readPendingAuthMethod()
			rememberStoredAccount(
				signedIn.value.user,
				signedIn.value.token,
				nextAuthMethod ? { authMethod: nextAuthMethod } : undefined,
			)
		}

		promotePendingSignInOAuthProvider()

		if (route.query.redirect) {
			await finishAuth(decodeURIComponent(getQueryString(route.query.redirect) ?? ''), true)
		} else if (signedIn.value.user) {
			await finishAuth(`/user/${signedIn.value.user.username}`, true)
		} else {
			await finishAuth()
		}
	}

	const showSignIn = computed(() => signInReady.value || launcherHandoff.value != null)

	async function loadAuth() {
		await useAuth()
	}

	async function redirectOAuthState() {
		if (route.query.state === undefined) {
			return
		}
		await navigateTo(
			{
				path: '/auth/create/oauth',
				query: route.query,
			},
			{
				replace: true,
			},
		)
	}

	async function finishSessionCode() {
		if (!route.query.code || isProtocolV2) {
			return
		}
		await finishSignIn()
	}

	const signIn = {
		showSignIn,
		launcherHandoff,
		flow,
		twoFactorCode,
		twoFactorPending,
		twoFactorError,
		accounts,
		reauthAccount: reauthAccountPreview,
		redirectTarget,
		routeQuery,
		globals,
		email,
		password,
		token,
		beginPasswordSignIn,
		begin2FASignIn,
		beginPasskeySignin,
		setCaptchaRef,
		cancelReauth,
		onSelectLauncherAccount,
	}

	provide(SIGN_IN_KEY, signIn)
	return {
		showSignIn,
		loadAuth,
		redirectOAuthState,
		finishSessionCode,
	}
}

export function useSignIn() {
	const signIn = inject(SIGN_IN_KEY)
	if (!signIn) {
		throw new Error('useSignIn() must be used within the sign-in screen')
	}
	return signIn
}
