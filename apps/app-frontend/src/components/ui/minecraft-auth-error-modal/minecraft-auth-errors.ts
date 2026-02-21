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
]
