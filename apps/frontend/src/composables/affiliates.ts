export const useAffiliates = () => {
	const affiliateCookie = useCookie('mrs_afl', {
		// maxAge: 60 * 60 * 24 * 7, // 7 days
		maxAge: 60 * 60, // an hour
		sameSite: 'lax',
		secure: true,
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
