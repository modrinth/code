export const BASE62_CHARS = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'
export type Base62Char = (typeof BASE62_CHARS)[number]

export type ModrinthId = string

export type Environment = 'required' | 'optional' | 'unsupported' | 'unknown'

export type RequestableStatus = 'approved' | 'archived' | 'unlisted' | 'private'
export type ApprovedStatus = RequestableStatus | 'scheduled'
export type UnapprovedStatus = 'draft' | 'processing' | 'rejected' | 'withheld'
export type ProjectStatus = ApprovedStatus | UnapprovedStatus | 'unknown'

export type DonationPlatform =
	| { short: 'patreon'; name: 'Patreon' }
	| { short: 'bmac'; name: 'Buy Me A Coffee' }
	| { short: 'paypal'; name: 'PayPal' }
	| { short: 'github'; name: 'GitHub Sponsors' }
	| { short: 'ko-fi'; name: 'Ko-fi' }
	| { short: 'other'; name: 'Other' }

export type ProjectType =
	| 'mod'
	| 'modpack'
	| 'resourcepack'
	| 'shader'
	| 'plugin'
	| 'datapack'
	| 'project'
export type MonetizationStatus = 'monetized' | 'demonetized' | 'force-demonetized'

export type GameVersion = string
export type Platform = string
export type Category = string
export type CategoryOrPlatform = Category | Platform

export interface DonationLink<T extends DonationPlatform> {
	id: T['short']
	platform: T['name']
	url: string
}

export interface GalleryImage {
	url: string
	featured: boolean
	created: string
	ordering: number

	title?: string
	description?: string
}

export interface ProjectV3 {
	id: ModrinthId
	slug?: string
	project_types: string[]
	games: string[]
	team_id: ModrinthId
	organization?: ModrinthId
	name: string
	summary: string
	description: string

	published: string
	updated: string
	approved?: string
	queued?: string

	status: ProjectStatus
	requested_status?: ProjectStatus

	/** @deprecated moved to threads system */
	moderator_message?: {
		message: string
		body?: string
	}

	license: {
		id: string
		name: string
		url?: string
	}

	downloads: number
	followers: number

	categories: string[]
	additional_categories: string[]
	loaders: string[]

	versions: ModrinthId[]
	icon_url?: string

	link_urls: Record<
		string,
		{
			platform: string
			donation: boolean
			url: string
		}
	>

	gallery: {
		url: string
		raw_url: string
		featured: boolean
		name?: string
		description?: string
		created: string
		ordering: number
	}[]

	color?: number
	thread_id: ModrinthId
	monetization_status: MonetizationStatus
	side_types_migration_review_status: EnvironmentMigrationReviewStatus

	[key: string]: unknown
}

export interface Project {
	id: ModrinthId
	project_type: ProjectType
	slug: string
	title: string
	description: string
	status: ProjectStatus
	requested_status: RequestableStatus
	monetization_status: MonetizationStatus

	body: string
	icon_url?: string
	color?: number

	categories: Category[]
	additional_categories: Category[]

	downloads: number
	followers: number

	client_side: Environment
	server_side: Environment

	team?: ModrinthId
	team_id: ModrinthId
	thread_id: ModrinthId
	organization: ModrinthId

	issues_url: string | null
	source_url: string | null
	wiki_url: string | null
	discord_url: string | null
	donation_urls: DonationLink<DonationPlatform>[]

	published: string
	created?: string
	updated: string
	approved: string
	queued: string

	game_versions: GameVersion[]
	loaders: Platform[]

	versions: ModrinthId[]
	gallery?: GalleryImage[]

	license: {
		id: string
		name: string
		url?: string
	}
}

export type EnvironmentMigrationReviewStatus = 'reviewed' | 'pending'
export type EnvironmentV3 =
	| 'client_and_server'
	| 'client_only'
	| 'client_only_server_optional'
	| 'singleplayer_only'
	| 'server_only'
	| 'server_only_client_optional'
	| 'dedicated_server_only'
	| 'client_or_server'
	| 'client_or_server_prefers_both'
	| 'unknown'

// This is only the fields we care about from v3, since we use v2 for the vast majority of project metadata.
export interface ProjectV3Partial {
	side_types_migration_review_status: EnvironmentMigrationReviewStatus
	environment: EnvironmentV3[]
	project_types: ProjectType[]
}

export interface SearchResult {
	id: ModrinthId
	project_type: ProjectType
	slug: string
	title: string
	description: string
	monetization_status: MonetizationStatus

	icon_url?: string
	color?: number

	categories: CategoryOrPlatform[]
	display_categories: CategoryOrPlatform[]
	versions: GameVersion[]
	latest_version: GameVersion

	downloads: number
	follows: number

	client_side: Environment
	server_side: Environment

	author: string

	date_created: string
	date_modified: string

	gallery: string[]
	featured_gallery?: string[]

	license: string
}

export type Organization = {
	id: ModrinthId
	slug: string
	name: string
	team_id: ModrinthId
	description: string
	icon_url: string
	color: number
	members: OrganizationMember[]
}

export type OrganizationPermissions = number

export type OrganizationMember = {
	team_id: ModrinthId
	user: User
	role: string
	is_owner: boolean
	permissions: TeamMemberPermissions
	organization_permissions: OrganizationPermissions
	accepted: boolean
	payouts_split: number
	ordering: number
}

export type Collection = {
	id: ModrinthId
	user: User
	name: string
	description: string
	icon_url: string
	color: number
	status: CollectionStatus
	created: string
	updated: string
	projects: ModrinthId[]
}

export type CollectionStatus = 'listed' | 'unlisted' | 'private' | 'unknown'

export type DependencyType = 'required' | 'optional' | 'incompatible' | 'embedded'

export interface VersionDependency {
	dependency_type: DependencyType
	file_name?: string
}

export interface ProjectDependency {
	dependency_type: DependencyType
	project_id?: string
}

export interface FileDependency {
	dependency_type: DependencyType
	file_name?: string
}

export type Dependency = VersionDependency | ProjectDependency | FileDependency
export type VersionChannel = 'release' | 'beta' | 'alpha'
export type VersionStatus = 'listed' | 'archived' | 'draft' | 'unlisted' | 'scheduled' | 'unknown'
export type FileType =
	| 'required-resource-pack'
	| 'optional-resource-pack'
	| 'sources-jar'
	| 'dev-jar'
	| 'javadoc-jar'
	| 'signature'
	| 'unknown'

export interface VersionFileHash {
	sha512: string
	sha1: string
}

export interface VersionFile {
	hashes: VersionFileHash
	url: string
	filename: string
	primary: boolean
	size: number
	file_type?: FileType
}

export interface Version {
	name: string
	version_number: string
	changelog?: string
	dependencies: Dependency[]
	game_versions: GameVersion[]
	version_type: VersionChannel
	loaders: Platform[]
	featured: boolean
	status: VersionStatus
	id: ModrinthId
	project_id: ModrinthId
	author_id: ModrinthId
	date_published: string
	downloads: number
	files: VersionFile[]
}

export interface PayoutData {
	balance: number
	payout_wallet: 'paypal' | 'venmo'
	payout_wallet_type: 'email' | 'phone' | 'user_handle'
	payout_address: string
}

export type UserRole = 'admin' | 'moderator' | 'pyro' | 'developer'

export enum UserBadge {
	MIDAS = 1 << 0,
	EARLY_MODPACK_ADOPTER = 1 << 1,
	EARLY_RESPACK_ADOPTER = 1 << 2,
	EARLY_PLUGIN_ADOPTER = 1 << 3,
	ALPHA_TESTER = 1 << 4,
	CONTRIBUTOR = 1 << 5,
	TRANSLATOR = 1 << 6,
	AFFILIATE = 1 << 7,
}

export type UserBadges = number

export interface User {
	username: string
	email?: string
	bio?: string
	payout_data?: PayoutData
	id: ModrinthId
	avatar_url: string
	created: string
	role: UserRole
	badges: UserBadges
	auth_providers?: string[]
	email_verified?: boolean
	has_password?: boolean
	has_totp?: boolean
}

export enum TeamMemberPermission {
	UPLOAD_VERSION = 1 << 0,
	DELETE_VERSION = 1 << 1,
	EDIT_DETAILS = 1 << 2,
	EDIT_BODY = 1 << 3,
	MANAGE_INVITES = 1 << 4,
	REMOVE_MEMBER = 1 << 5,
	EDIT_MEMBER = 1 << 6,
	DELETE_PROJECT = 1 << 7,
	VIEW_ANALYTICS = 1 << 8,
	VIEW_PAYOUTS = 1 << 9,
}

export type TeamMemberPermissions = number

export interface TeamMember {
	team_id: ModrinthId
	user: User
	role: string
	permissions: TeamMemberPermissions
	accepted: boolean
	payouts_split: number
	ordering: number
	is_owner: boolean
}

export type Report = {
	id: ModrinthId
	item_id: ModrinthId
	item_type: 'project' | 'version' | 'user'
	report_type: string
	reporter: ModrinthId
	thread_id: ModrinthId
	closed: boolean
	created: string
	body: string
}

// Threads
export interface Thread {
	id: string
	type: ThreadType
	project_id: string | null
	report_id: string | null
	messages: ThreadMessage[]
	members: User[]
}

export type ThreadType = 'project' | 'report' | 'direct_message'

export interface ThreadMessage {
	id: string | null
	author_id: string | null
	body: MessageBody
	created: string
	hide_identity: boolean
}

export type MessageBody =
	| TextMessageBody
	| StatusChangeMessageBody
	| ThreadClosureMessageBody
	| ThreadReopenMessageBody
	| DeletedMessageBody

export interface TextMessageBody {
	type: 'text'
	body: string
	private: boolean
	replying_to: string | null
	associated_images: string[]
}

export interface StatusChangeMessageBody {
	type: 'status_change'
	new_status: ProjectStatus
	old_status: ProjectStatus
}

export interface ThreadClosureMessageBody {
	type: 'thread_closure'
}

export interface ThreadReopenMessageBody {
	type: 'thread_reopen'
}

export interface DeletedMessageBody {
	type: 'deleted'
	private: boolean
}

// Moderation
export interface ModerationModpackPermissionApprovalType {
	id:
		| 'yes'
		| 'no'
		| 'with-attribution'
		| 'unidentified'
		| 'with-attribution-and-source'
		| 'permanent-no'
	name: string
}

export interface ModerationPermissionType {
	id: 'yes' | 'no'
	name: string
}

export interface ModerationBaseModpackItem {
	sha1: string
	file_name: string
	type: 'unknown' | 'flame' | 'identified'
	status: ModerationModpackPermissionApprovalType['id'] | null
	approved: ModerationPermissionType['id'] | null
}

export interface ModerationUnknownModpackItem extends ModerationBaseModpackItem {
	type: 'unknown'
	proof: string
	url: string
	title: string
}

export interface ModerationFlameModpackItem extends ModerationBaseModpackItem {
	type: 'flame'
	id: string
	title: string
	url: string
}

export interface ModerationIdentifiedModpackItem extends ModerationBaseModpackItem {
	type: 'identified'
	proof?: string
	url?: string
	title?: string
}

export type ModerationModpackItem =
	| ModerationUnknownModpackItem
	| ModerationFlameModpackItem
	| ModerationIdentifiedModpackItem

export interface ModerationModpackResponse {
	identified?: Record<
		string,
		{
			file_name: string
			status: ModerationModpackPermissionApprovalType['id']
		}
	>
	unknown_files?: Record<string, string>
	flame_files?: Record<
		string,
		{
			file_name: string
			id: string
			title?: string
			url?: string
		}
	>
}

export interface ModerationJudgement {
	type: 'flame' | 'unknown' | 'identified'
	status: string | null
	id?: string
	link?: string
	title?: string
	proof?: string
	file_name?: string
}

export interface ModerationJudgements {
	[sha1: string]: ModerationJudgement
}

// Subscriptions
export interface UserSubscription {
	id: string
	user_id: string
	price_id: string
	interval: 'five-days' | 'monthly' | 'quarterly' | 'yearly'
	status: 'provisioned' | 'unprovisioned'
	created: string // ISO date string
	metadata?: SubscriptionMetadata
}

export interface Charge {
	id: string
	user_id: string
	price_id: string
	amount: number
	currency_code: string
	status: 'open' | 'processing' | 'succeeded' | 'failed' | 'cancelled' | 'expiring'
	due: string // ISO date string
	last_attempt?: string // ISO date string
	type: 'one-time' | 'subscription' | 'proration' | 'refund'
	subscription_id?: string
	subscription_interval?: 'five-days' | 'monthly' | 'quarterly' | 'yearly'
	platform: 'stripe' | 'none'
	parent_charge_id?: string
	net?: number
}

export type SubscriptionMetadata =
	| { type: 'pyro'; id: string; region?: string }
	| { type: 'medal'; id: string }

// Delphi
export interface DelphiReport {
	id: string
	project: Project
	version: Version
	priority_score: number
	detected_at: string
	trace_type:
		| 'reflection_indirection'
		| 'xor_obfuscation'
		| 'included_libraries'
		| 'suspicious_binaries'
		| 'corrupt_classes'
		| 'suspicious_classes'
		| 'url_usage'
		| 'classloader_usage'
		| 'processbuilder_usage'
		| 'runtime_exec_usage'
		| 'jni_usage'
		| 'main_method'
		| 'native_loading'
		| 'malformed_jar'
		| 'nested_jar_too_deep'
		| 'failed_decompilation'
		| 'analysis_failure'
		| 'malware_easyforme'
		| 'malware_simplyloader'
	file_path: string
	// pending = not reviewed yet.
	// approved = approved as malicious, removed from modrinth
	// rejected = not approved as malicious, remains on modrinth?
	status: 'pending' | 'approved' | 'rejected'
	content?: string
}

export type PayoutId = string
export type UserId = string
export type PayoutStatus =
	| 'success'
	| 'in-transit'
	| 'cancelled'
	| 'cancelling'
	| 'failed'
	| 'unknown'
export type PayoutMethodType = 'venmo' | 'paypal' | 'tremendous' | 'muralpay' | 'unknown'

export interface Payout {
	id: PayoutId
	user_id: UserId
	status: PayoutStatus
	created: string // ISO 8601
	amount: number
	fee: number | null
	method: PayoutMethodType | null
	method_address: string | null
	platform_id: string | null
}

export type PayoutList = Payout[]

// Revenue event types for transaction history
export interface IncomeEvent {
	type: 'payout_available'
	created: string // ISO 8601
	payout_source: string
	amount: number
}

export interface WithdrawalEvent {
	type: 'withdrawal'
	id: string
	status: PayoutStatus
	created: string // ISO 8601
	amount: number
	fee: number | null
	method_type: PayoutMethodType | null
	method_address: string | null
}

export type RevenueEvent = IncomeEvent | WithdrawalEvent

export type RevenueEventList = RevenueEvent[]

export interface PayoutMethodFee {
	percentage: number
	min: number
	max: number | null
}

export type PayoutInterval =
	| {
			type: 'standard'
			min: number
			max: number
	  }
	| {
			type: 'fixed'
			values: number[]
	  }

export interface PayoutMethod {
	id: string
	type: PayoutMethodType
	name: string
	supported_countries: string[]
	image_url: string | null
	interval: PayoutInterval
	fee: PayoutMethodFee
}

export type AffiliateLink = {
	id: string
	created_at: string
	created_by: string
	affiliate: string
	source_name: string
}
