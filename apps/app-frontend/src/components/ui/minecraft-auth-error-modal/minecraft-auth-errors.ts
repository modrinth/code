export interface MinecraftAuthError {
	errorCode: string
	whatHappened: string
	stepsToFix: string[]
}

export const minecraftAuthErrors: MinecraftAuthError[] = [
	{
		errorCode: '2148916222',
		whatHappened:
			'Your Minecraft/Xbox Live account requires age verification to comply with UK regulations. You must complete this before signing in.',
		stepsToFix: [
			'Go to the <a href="https://www.minecraft.net/en-us/login">Minecraft Login</a> page and sign in',
			'Follow the instructions to verify your age',
			'Once verified, try signing in again',
			'For additional help, visit <a href="https://aka.ms/XboxUKAgeSupport">Xbox UK Age Support</a>',
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
]
