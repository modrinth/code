import type { ReportQuickReply } from '../types/reports'

export default [
  {
    label: 'Test Quick Reply',
    message: 'This is a test quick reply message.',
    shouldShow: (report) => report.project?.project_type === 'mod',
    private: false,
  },
] as ReadonlyArray<ReportQuickReply>
