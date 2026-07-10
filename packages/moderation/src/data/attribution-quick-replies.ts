import type { QuickReply } from '../types/quick-reply'

export default [
	{
		label: '✅ Corrections Applied',
		message: async () =>
			(await import('./messages/quick-replies/externals-permissions/corrections-applied.md?raw'))
				.default,
		private: false,
	},
	{
		label: '🚫 Bad Proofs',
		message: async () =>
			(await import('./messages/quick-replies/externals-permissions/illegitimate-evidence.md?raw'))
				.default,
		private: false,
	},
	{
		label: '⛔ Inaccessible Proofs',
		message: async () =>
			(await import('./messages/quick-replies/externals-permissions/inaccessible-evidence.md?raw'))
				.default,
		private: false,
	},
	{
		label: '🌐 Not Permission to Distribute',
		message: async () =>
			(await import('./messages/quick-replies/externals-permissions/but-its-online.md?raw'))
				.default,
		private: false,
	},
	{
		label: '📝 ARR License',
		message: async () =>
			(await import('./messages/quick-replies/externals-permissions/arr-license.md?raw')).default,
		private: false,
	},
	{
		label: '💰 Non-Commercial License',
		message: async () =>
			(await import('./messages/quick-replies/externals-permissions/nc-license.md?raw')).default,
		private: false,
	},
	{
		label: '🍴 Forks',
		message: async () =>
			(await import('./messages/quick-replies/externals-permissions/forks.md?raw')).default,
		private: false,
	},
	{
		label: '💲 Premium Content',
		message: async () =>
			(await import('./messages/quick-replies/externals-permissions/premium-content.md?raw'))
				.default,
		private: false,
	},
	{
		label: '⚖️ Prohibited Content',
		message: async () =>
			(await import('./messages/quick-replies/externals-permissions/prohibited-content.md?raw'))
				.default,
		private: false,
	},
	{
		label: '🧑‍💻 Cheats and Hacks',
		message: async () =>
			(await import('./messages/quick-replies/externals-permissions/cheats-and-hacks.md?raw'))
				.default,
		private: false,
	},
] as ReadonlyArray<QuickReply>
