import { invoke } from '@tauri-apps/api/core'

export function getRecommendedJavaVersion(instanceId: string): Promise<number> {
	return invoke('plugin:instance|instance_get_recommended_java_version', { instanceId })
}

export type ContentListFormat = 'detailed' | 'names' | 'file-names' | 'urls' | 'markdown'

export interface ContentListOptions {
	format: ContentListFormat
}

export function getContentList(instanceId: string, options: ContentListOptions): Promise<string> {
	return invoke('plugin:instance|instance_get_content_list', { instanceId, options })
}
