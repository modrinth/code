import { invoke } from '@tauri-apps/api/core'

export function getRecommendedJavaVersion(instanceId: string): Promise<number> {
	return invoke('plugin:instance|instance_get_recommended_java_version', { instanceId })
}
