export type ActionKind = 'game-command' | 'restart'

export type ScheduleOptions = { command: string } | Record<string, never>

export interface Schedule {
  title: string
  every: string
  action_kind: ActionKind
  options: ScheduleOptions
  enabled: boolean
  warn_msg: string
  warn_intervals: number[]
}

export interface ServerSchedule extends Schedule {
  id: number
  server_id: string
  added_on: string
}
