export interface ScheduledTask {
  id?: number
  title: string
  action_kind: 'game-command' | 'restart'
  options: { command?: string }
  enabled: boolean
  warn_msg?: string
  warn_intervals: number[]
  every: string
}
