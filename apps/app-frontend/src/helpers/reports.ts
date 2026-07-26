import { invoke } from '@tauri-apps/api/core'

export type ReportItemType = 'project' | 'version' | 'user' | 'shared-instance'

export interface CreateReportRequest {
	report_type: string
	item_id: string
	item_type: ReportItemType
	body: string
	uploaded_images: string[]
}

export interface CreateReportResponse {
	id: string
}

export async function create_report(request: CreateReportRequest) {
	return await invoke<CreateReportResponse>('plugin:reports|reports_create', { request })
}
