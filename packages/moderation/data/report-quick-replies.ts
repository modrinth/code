import type { ReportQuickReply } from '../types/reports'

export default [
  {
    label: 'Antivirus',
    message: async () => (await import('./messages/reports/antivirus.md?raw')).default,
    private: false,
  },
  {
    label: 'Spam',
    message: async () => (await import('./messages/reports/spam.md?raw')).default,
    private: false,
  },
  {
    label: 'Gameplay Issue',
    message: async () => (await import('./messages/reports/gameplay-issue.md?raw')).default,
    private: false,
  },
  {
    label: 'Platform Issue',
    message: async () => (await import('./messages/reports/platform-issue.md?raw')).default,
    private: false,
  },
  {
    label: 'Stale',
    message: async () => (await import('./messages/reports/stale.md?raw')).default,
    private: false,
  },
  {
    label: 'Confirmed Malware',
    message: async () => (await import('./messages/reports/confirmed-malware.md?raw')).default,
    private: false,
  },
] as ReadonlyArray<ReportQuickReply>
