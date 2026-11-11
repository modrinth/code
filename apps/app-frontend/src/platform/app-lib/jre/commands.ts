import { invoke } from '@tauri-apps/api/core'

import type { JavaVersion } from './types'

export function getJavaVersions(): Promise<Record<number, JavaVersion>> {
	return invoke('plugin:jre|get_java_versions')
}

export function setJavaVersion(javaVersion: Partial<JavaVersion>): Promise<void> {
	return invoke('plugin:jre|set_java_version', { javaVersion })
}

export function findFilteredJres(version: number | null): Promise<JavaVersion[]> {
	return invoke('plugin:jre|jre_find_filtered_jres', { version })
}

export function getJre(path: string): Promise<JavaVersion | null> {
	return invoke('plugin:jre|jre_get_jre', { path })
}

export function autoInstallJava(javaVersion: number): Promise<string> {
	return invoke('plugin:jre|jre_auto_install_java', { javaVersion })
}

export function getMaxMemory(): Promise<number> {
	return invoke('plugin:jre|jre_get_max_memory')
}
