import type { QuickReply } from '../types/quick-reply'

export default [
	{
		label: '⚠️ Illegitimate Evidence',
		message: async () =>
			(await import('./messages/attributions/illegitimate-evidence.md?raw')).default,
		private: false,
	},
	{
		label: '🚫 Cheats and Hacks',
		message: async () => (await import('./messages/attributions/cheats-and-hacks.md?raw')).default,
		private: false,
	},
] as ReadonlyArray<QuickReply>
