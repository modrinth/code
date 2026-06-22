import type { QuickReply } from '../types/quick-reply'
import type { ExtendedReport } from '../types/reports'

export default [
	{
		label: '✅⚖️ Action Taken',
		message: async () =>
			(await import('./messages/quick-replies/reports/action-taken.md?raw')).default,
		private: false,
	},
	{
		label: '✅⚠️ Author Alerted',
		message: async () =>
			(await import('./messages/quick-replies/reports/author-alerted.md?raw')).default,
		private: false,
	},
	{
		label: '✅🏷️ Metadata Corrected',
		message: async () =>
			(await import('./messages/quick-replies/reports/metadata-corrected.md?raw')).default,
		private: false,
	},
	{
		label: '❌🏷️ Metadata Insufficient',
		message: async () =>
			(await import('./messages/quick-replies/reports/metadata-insufficient.md?raw')).default,
		private: false,
	},
	{
		label: '❌📤 Reupload Insufficient',
		message: async () =>
			(await import('./messages/quick-replies/reports/reupload-insufficient.md?raw')).default,
		private: false,
	},
	{
		label: '❌🏳️‍🌈 Spam',
		message: async () => (await import('./messages/quick-replies/reports/spam.md?raw')).default,
		private: false,
	},
	{
		label: '❌🛡️ Antivirus',
		message: async () =>
			(await import('./messages/quick-replies/reports/antivirus.md?raw')).default,
		private: false,
	},
	{
		label: '❌👻 Functionality Disclosed',
		message: async () =>
			(await import('./messages/quick-replies/reports/functionality-disclosed.md?raw')).default,
		private: false,
	},
	{
		label: '❌🤖 Suspected AI',
		message: async () =>
			(await import('./messages/quick-replies/reports/suspected-ai.md?raw')).default,
		private: false,
	},
	{
		label: '❌👩‍💻 Not a Hack',
		message: async () =>
			(await import('./messages/quick-replies/reports/not-a-hack.md?raw')).default,
		private: false,
	},
	{
		label: '❌🕹️ Gameplay Issue',
		message: async () =>
			(await import('./messages/quick-replies/reports/gameplay-issue.md?raw')).default,
		private: false,
	},
	{
		label: '❌🎧 Platform Issue',
		message: async () =>
			(await import('./messages/quick-replies/reports/platform-issue.md?raw')).default,
		private: false,
	},
	{
		label: '❌🪦 Stale',
		message: async () => (await import('./messages/quick-replies/reports/stale.md?raw')).default,
		private: false,
	},
] as ReadonlyArray<QuickReply<ExtendedReport>>
