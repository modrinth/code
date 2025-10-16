import type { ReportQuickReply } from '../types/reports'

export default [
	{
		label: '✅⚖️ Action Taken',
		message: async () => (await import('./messages/reports/action-taken.md?raw')).default,
		private: false,
	},
	{
		label: '✅⚠️ Author Alerted',
		message: async () => (await import('./messages/reports/author-alerted.md?raw')).default,
		private: false,
	},
	{
		label: '✅🏷️ Metadata Corrected',
		message: async () => (await import('./messages/reports/metadata-corrected.md?raw')).default,
		private: false,
	},
	{
		label: '❌🏷️ Metadata Insufficient',
		message: async () => (await import('./messages/reports/metadata-insufficient.md?raw')).default,
		private: false,
	},
	{
		label: '❌📤 Reupload Insufficient',
		message: async () => (await import('./messages/reports/reupload-insufficient.md?raw')).default,
		private: false,
	},
	{
		label: '❌🏳️‍🌈 Spam',
		message: async () => (await import('./messages/reports/spam.md?raw')).default,
		private: false,
	},
	{
		label: '❌🛡️ Antivirus',
		message: async () => (await import('./messages/reports/antivirus.md?raw')).default,
		private: false,
	},
	{
		label: '❌👻 Functionality Disclosed',
		message: async () =>
			(await import('./messages/reports/functionality-disclosed.md?raw')).default,
		private: false,
	},
	{
		label: '❌🤖 Suspected AI',
		message: async () => (await import('./messages/reports/suspected-ai.md?raw')).default,
		private: false,
	},
	{
		label: '❌👩‍💻 Not a Hack',
		message: async () => (await import('./messages/reports/not-a-hack.md?raw')).default,
		private: false,
	},
	{
		label: '❌🕹️ Gameplay Issue',
		message: async () => (await import('./messages/reports/gameplay-issue.md?raw')).default,
		private: false,
	},
	{
		label: '❌🎧 Platform Issue',
		message: async () => (await import('./messages/reports/platform-issue.md?raw')).default,
		private: false,
	},
	{
		label: '❌🪦 Stale',
		message: async () => (await import('./messages/reports/stale.md?raw')).default,
		private: false,
	},
] as ReadonlyArray<ReportQuickReply>
