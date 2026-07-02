export interface MinecraftAuthError {
	errorCode?: string
	errorMatchers?: string[]
	matches?: (message: string) => boolean
	whatHappened: string
	stepsToFix: string[]
}

export const minecraftAuthErrors: MinecraftAuthError[] = [
	{
		errorMatchers: ['Failed to deserialize response to JSON during step RefreshOAuthToken:'],
		whatHappened:
			'Your saved Microsoft sign-in token has expired or was revoked, so Modrinth App cannot refresh your Minecraft session.',
		stepsToFix: [
			'Sign out of the affected Minecraft account in Modrinth App',
			'Sign in to the account again',
			'Once the new sign-in finishes, try launching Minecraft again',
		],
	},
	{
		errorMatchers: ['Failed to deserialize response to JSON during step SisuAuthenticate:'],
		whatHappened:
			'Xbox services rejected the first sign-in response. This is most often caused by your system clock or time zone being out of sync.',
		stepsToFix: [
			'Open your system date and time settings',
			'Turn on automatic time zone and automatic time, if available',
			'Use the sync option in your system settings to synchronize the clock',
			'Restart Modrinth App',
			'Try signing in again',
		],
	},
	{
		matches: (message) =>
			message.includes('Failed to deserialize response to JSON during step MinecraftToken:') &&
			message.includes('429 Too Many Requests'),
		whatHappened:
			'Microsoft or Minecraft temporarily blocked the sign-in request because there were too many recent attempts.',
		stepsToFix: [
			'Wait about an hour before trying again',
			'Restart Modrinth App after waiting',
			'Try signing in once more',
			'If the same message appears, wait longer before retrying so the temporary limit can clear',
		],
	},
	{
		matches: (message) =>
			message.includes('Failed to deserialize response to JSON during step MinecraftToken:') &&
			/Status Code: 5\d\d/.test(message),
		whatHappened:
			"Minecraft's authentication service is returning a server error, so Modrinth App cannot finish signing you in right now.",
		stepsToFix: [
			'Wait a few minutes and try signing in again',
			'Check <a href="https://support.xbox.com/xbox-live-status">Xbox Status</a> for current service issues',
			'Try signing in with the <a href="https://www.minecraft.net/en-us/download">official Minecraft Launcher</a> to confirm whether Minecraft sign-in is also affected there',
			'If the service is healthy and this keeps happening, contact support with the debug information below',
		],
	},
	{
		errorMatchers: ['Failed to fetch player profile'],
		whatHappened:
			'Minecraft services could not return a Java Edition profile for this account. This most often happens when the game was purchased recently, the Java profile has not finished being created, or the wrong Microsoft account is being used.',
		stepsToFix: [
			'Sign in with the <a href="https://www.minecraft.net/en-us/download">official Minecraft Launcher</a>',
			'Launch Minecraft: Java Edition once from the official launcher',
			'Wait up to an hour if the purchase or profile setup was recent',
			'Make sure you are using the Microsoft account that owns Minecraft. See <a href="https://support.modrinth.com/en/articles/9409136-finding-the-right-xbox-account">Finding the right Xbox account</a> for help',
			'Try signing in to Modrinth App again',
		],
	},
	{
		matches: (message) =>
			message.includes('error sending request for url (') &&
			[
				'minecraft.net',
				'minecraftservices.com',
				'mojang.com',
				'xbox.com',
				'xboxlive.com',
				'live.com',
			].some((domain) => message.includes(domain)),
		whatHappened:
			'Modrinth App could not connect to a Microsoft, Xbox, or Minecraft service needed for sign-in. This is usually caused by a local network, DNS, proxy, firewall, hosts file, VPN, or antivirus issue.',
		stepsToFix: [
			'Restart Modrinth App and try signing in again',
			'Check that your internet connection is working',
			'Allow Modrinth App through your firewall, antivirus, proxy, VPN, and hosts file rules',
			'Try a different network or temporarily disable VPN/proxy software if you use one',
			'If routing or DNS is the issue, a service like Cloudflare WARP can sometimes help',
		],
	},
	{
		errorCode: '2148916222',
		whatHappened:
			'Your Minecraft/Xbox Live account requires age verification to comply with UK regulations. You must complete this before signing in.',
		stepsToFix: [
			'Go to the <a href="https://www.minecraft.net/en-us/login">Minecraft Login</a> page and sign in',
			'Follow the instructions to verify your age',
			'Once verified, try signing in again',
			'For additional help, visit <a href="https://support.xbox.com/en-GB/help/family-online-safety/online-safety/UK-age-verification">UK age verification on Xbox</a>',
		],
	},
	{
		errorCode: '2148916233',
		whatHappened: "This account doesn't have an Xbox profile set up or doesn't own Minecraft.",
		stepsToFix: [
			'Make sure Minecraft is purchased on this account',
			'Visit <a href="https://www.minecraft.net/en-us/login">Minecraft Login</a> and sign in',
			'Complete Xbox profile setup if prompted',
			'Once finished, try signing in again',
		],
	},
	{
		errorCode: '2148916235',
		whatHappened: "Xbox Live isn't available in your region, so sign-in is blocked.",
		stepsToFix: [
			'Xbox services must be supported in your country before you can sign in',
			'Check <a href="https://www.xbox.com/en-US/regions">Xbox Availability</a> for supported regions',
		],
	},
	{
		errorCode: '2148916236',
		whatHappened: 'This account requires adult verification under South Korean regulations.',
		stepsToFix: [
			'Visit <a href="https://www.xbox.com">Xbox</a> and sign in',
			'Complete the identity verification process',
			'Once finished, try signing in again',
		],
	},
	{
		errorCode: '2148916237',
		whatHappened: 'This account requires adult verification under South Korean regulations.',
		stepsToFix: [
			'Visit <a href="https://www.xbox.com">Xbox</a> and sign in',
			'Complete the identity verification process',
			'Once finished, try signing in again',
		],
	},
	{
		errorCode: '2148916238',
		whatHappened: 'This account is underage and not linked to a Microsoft family group.',
		stepsToFix: [
			'Review the <a href="https://help.minecraft.net/hc/en-us/articles/4408968616077">Family Setup Guide</a>',
			'Join or create a family group as instructed',
			'Once finished, try signing in again',
		],
	},
	{
		errorCode: '2148916227',
		whatHappened: 'This account was suspended for violating Xbox Community Standards.',
		stepsToFix: [
			'Visit <a href="https://support.xbox.com">Xbox Support</a> and review the enforcement details',
			'Submit an appeal if one is available',
		],
	},
	{
		errorCode: '2148916229',
		whatHappened: "This account is restricted and doesn't have permission to play online.",
		stepsToFix: [
			'Have a guardian sign in to <a href="https://account.microsoft.com/family/">Microsoft Family</a>',
			'Update online play permissions',
			'Once finished, try signing in again',
		],
	},
	{
		errorCode: '2148916234',
		whatHappened: "This account hasn't accepted Xbox's Terms of Service.",
		stepsToFix: [
			'Visit <a href="https://www.xbox.com">Xbox</a> and sign in',
			'Accept the Terms if prompted',
			'Once finished, try signing in again',
		],
	},
	{
		errorMatchers: ['Failed to deserialize response to JSON during step XstsAuthorize:'],
		whatHappened:
			'Xbox services rejected the request to authorize this account for Minecraft services, but did not return a specific account restriction that Modrinth App recognizes.',
		stepsToFix: [
			'Sign in with the <a href="https://www.minecraft.net/en-us/download">official Minecraft Launcher</a>',
			'Complete any prompts shown by Microsoft, Xbox, or Minecraft',
			'Try signing in to Modrinth App again',
			'If the official launcher also fails, follow the error shown there or contact Xbox Support',
		],
	},
]

export function findMinecraftAuthError(message: string): MinecraftAuthError | null {
	return (
		minecraftAuthErrors.find((error) => {
			if (error.errorCode && message.includes(error.errorCode)) {
				return true
			}

			if (error.errorMatchers?.some((matcher) => message.includes(matcher))) {
				return true
			}

			return error.matches?.(message) ?? false
		}) ?? null
	)
}
