import type {
	DonationLink,
	DonationPlatform,
	Environment,
	EnvironmentMigrationReviewStatus,
	EnvironmentV3,
	Project,
	ProjectStatus,
	ProjectV3Partial,
	RequestableStatus,
} from '../types'

export type ProjectEditBody = {
	slug?: string
	title?: string
	description?: string
	categories?: string[]
	client_side?: Environment
	server_side?: Environment
	status?: ProjectStatus
	requested_status?: RequestableStatus
	additional_categories?: string[]
	issues_url?: string
	source_url?: string
	wiki_url?: string
	discord_url?: string
	donation_urls?: DonationLink<DonationPlatform>[]
	license_id?: string
	license_url?: string
}
export type ProjectV3EditBodyPartial = {
	environment?: EnvironmentV3
	side_types_migration_review_status: EnvironmentMigrationReviewStatus
}

export interface ModrinthApiProjects {
	get(id: string): Promise<Project>
	getV3(id: string): Promise<ProjectV3Partial>
	edit(id: string, data: ProjectEditBody): Promise<void>
	editV3(id: string, data: ProjectV3EditBodyPartial): Promise<void>
}
