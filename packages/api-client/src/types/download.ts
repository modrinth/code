import type { RequestOptions } from './request'

export type DownloadSink = (url: string, options: RequestOptions) => Promise<void>
