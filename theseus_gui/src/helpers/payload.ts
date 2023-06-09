export type PayloadCallback<T, R> = (payload: T) => R

export interface LoadingPayload {
  event: {
    /** One of  "StateInit", "PackDownload", etc */
    type: string
    /** The name of the pack */
    pack_name: string
    /** The id of the modpack */
    pack_id?: string
    /** The version of the modpack */
    pack_version?: string
    /** The name of the profile */
    profile_name: string
    /** A unique identification of the profile */
    profile_uuid: string
  }
  /** A unique identification of the loading bar */
  loader_uuid: string
  /** As a fraction of 1, how much we've loaded so far. If null, by convention, loading is finished */
  fraction: number | null
  /** A message to display to the user */
  message: string
}

export interface ProcessPayload {
  /** A unique identification of the process in the state (currently identified by PID, but that will change) */
  uuid: string
  /** The process ID */
  pid: number
  /** The event type */
  event: 'Launched' | 'Finished'
  /** A message to display to the user */
  message: string
}

export interface ProfilePayload {
  /** A unique identification of the process in the state (currently identified by path, but that will change) */
  uuid: string
  /** The name of the profile */
  name: string
  /** The path to profile */
  path: string
  /** The event type */
  event: 'Created' | 'Added' | 'Edited' | 'Removed'
}

export interface WarningPayload {
  /** A message to display to the user */
  message: string
}
