import type { ModrinthServerError } from '../errors'

export interface V1ErrorInfo {
	context?: string
	error: string
	description: string
}

export interface JWTAuth {
	url: string
	token: string
}

export interface ModuleError {
	error: ModrinthServerError
	timestamp: number
}

export type ModuleName = 'general' | 'content' | 'backups' | 'network' | 'startup' | 'ws' | 'fs'
