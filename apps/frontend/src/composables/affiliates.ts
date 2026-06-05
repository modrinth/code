export const useAffiliates = () => {
	const config = useRuntimeConfig()
	const affiliateCookie = useCookie('mrs_afl', {
		maxAge: 60 * 60 * 24 * 7, // 7 days
		sameSite: 'lax',
		secure: config.public.cookieSecure,
		httpOnly: false,
		path: '/',
	})

	const setAffiliateCode = (code: string) => {
		affiliateCookie.value = code
	}

	const getAffiliateCode = (): string | undefined => {
		return affiliateCookie.value || undefined
	}

	return {
		setAffiliateCode,
		getAffiliateCode,
	}
}
