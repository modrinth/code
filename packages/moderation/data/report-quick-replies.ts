import type { ReportQuickReply } from '../types/reports'

export default [
  {
    label: 'Antivirus',
    message: (await import('./messages/reports/antivirus.md?raw')).default,
    private: false,
  },
  {
    label: 'Spam',
    message: (await import('./messages/reports/spam.md?raw')).default,
    private: false,
  },
  {
    label: 'Gameplay Issue',
    message: (await import('./messages/reports/gameplay-issue.md?raw')).default,
    private: false,
  },
  {
    label: 'Platform Issue',
    message: (await import('./messages/reports/platform-issue.md?raw')).default,
    private: false,
  },
  {
    label: 'Stale',
    message: (await import('./messages/reports/stale.md?raw')).default,
    private: false,
  },
  {
    label: 'Confirmed Malware',
    message: (await import('./messages/reports/confirmed-malware.md?raw')).default,
    private: false,
  },
] as ReadonlyArray<ReportQuickReply>
