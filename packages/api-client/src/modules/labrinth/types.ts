import type { ISO3166 } from '../iso3166/types'

export namespace Labrinth {
	export namespace Billing {
		export namespace Internal {
			export type PriceDuration = 'five-days' | 'monthly' | 'quarterly' | 'yearly'

			export type SubscriptionStatus = 'provisioned' | 'unprovisioned'

			export type UserSubscription = {
				id: string
				user_id: string
				price_id: string
				interval: PriceDuration
				status: SubscriptionStatus
				created: string
				metadata?: SubscriptionMetadata
			}

			export type SubscriptionMetadata =
				| { type: 'pyro'; id: string; region?: string }
				| { type: 'medal'; id: string }

			export type ChargeStatus =
				| 'open'
				| 'processing'
				| 'succeeded'
				| 'failed'
				| 'cancelled'
				| 'expiring'

			export type ChargeType = 'one-time' | 'subscription' | 'proration' | 'refund'

			export type PaymentPlatform = 'Stripe' | 'None'

			export type Charge = {
				id: string
				user_id: string
				price_id: string
				amount: number
				currency_code: string
				status: ChargeStatus
				due: string
				last_attempt: string | null
				type: ChargeType
				subscription_id: string | null
				subscription_interval: PriceDuration | null
				platform: PaymentPlatform
				parent_charge_id: string | null
				net: number | null
			}

			export type ProductMetadata =
				| { type: 'midas' }
				| {
						type: 'pyro'
						cpu: number
						ram: number
						swap: number
						storage: number
				  }
				| {
						type: 'medal'
						cpu: number
						ram: number
						swap: number
						storage: number
						region: string
				  }

			export type Price =
				| { type: 'one-time'; price: number }
				| { type: 'recurring'; intervals: Record<PriceDuration, number> }

			export type ProductPrice = {
				id: string
				product_id: string
				prices: Price
				currency_code: string
			}

			export type Product = {
				id: string
				metadata: ProductMetadata
				prices: ProductPrice[]
				unitary: boolean
			}

			export type EditSubscriptionRequest = {
				interval?: PriceDuration
				payment_method?: string
				cancelled?: boolean
				region?: string
				product?: string
			}

			export type EditSubscriptionResponse = {
				payment_intent_id: string
				client_secret: string
				tax: number
				total: number
			}

			export type AddPaymentMethodFlowResponse = {
				client_secret: string
			}

			export type EditPaymentMethodRequest = {
				primary: boolean
			}

			export type InitiatePaymentRequest = {
				type: 'payment_method' | 'confirmation_token'
				id?: string
				token?: string
				charge:
					| { type: 'existing'; id: string }
					| { type: 'new'; product_id: string; interval?: PriceDuration }
				existing_payment_intent?: string
				metadata?: {
					type: 'pyro'
					server_name?: string
					server_region?: string
					source: unknown
				}
			}

			export type InitiatePaymentResponse = {
				payment_intent_id?: string
				client_secret?: string
				price_id: string
				tax: number
				total: number
				payment_method?: string
			}

			export type RefundChargeRequest = {
				type: 'full' | 'partial' | 'none'
				amount?: number
				unprovision?: boolean
			}

			export type CreditRequest =
				| { subscription_ids: string[]; days: number; send_email: boolean; message: string }
				| { nodes: string[]; days: number; send_email: boolean; message: string }
				| { region: string; days: number; send_email: boolean; message: string }
		}
	}

	export namespace Projects {
		export namespace v2 {
			export type Environment = 'required' | 'optional' | 'unsupported' | 'unknown'

			export type ProjectStatus =
				| 'approved'
				| 'archived'
				| 'rejected'
				| 'draft'
				| 'unlisted'
				| 'processing'
				| 'withheld'
				| 'scheduled'
				| 'private'
				| 'unknown'

			export type MonetizationStatus = 'monetized' | 'demonetized' | 'force-demonetized'

			export type ProjectType =
				| 'mod'
				| 'modpack'
				| 'resourcepack'
				| 'shader'
				| 'plugin'
				| 'datapack'
				| 'project'

			export type GalleryImage = {
				url: string
				featured: boolean
				title?: string
				description?: string
				created: string
				ordering: number
			}

			export type DonationLink = {
				id: string
				platform: string
				url: string
			}

			export type Project = {
				id: string
				slug: string
				project_type: ProjectType
				team: string
				title: string
				description: string
				body: string
				published: string
				updated: string
				approved?: string
				queued?: string
				status: ProjectStatus
				requested_status?: ProjectStatus
				moderator_message?: {
					message: string
					body?: string
				}
				license: {
					id: string
					name: string
					url?: string
				}
				client_side: Environment
				server_side: Environment
				downloads: number
				followers: number
				categories: string[]
				additional_categories: string[]
				game_versions: string[]
				loaders: string[]
				versions: string[]
				icon_url?: string
				issues_url?: string
				source_url?: string
				wiki_url?: string
				discord_url?: string
				donation_urls?: DonationLink[]
				gallery?: GalleryImage[]
				color?: number
				thread_id: string
				monetization_status: MonetizationStatus
			}

			export type SearchResultHit = {
				project_id: string
				project_type: ProjectType
				slug: string
				author: string
				title: string
				description: string
				categories: string[]
				display_categories: string[]
				versions: string[]
				downloads: number
				follows: number
				icon_url: string
				date_created: string
				date_modified: string
				latest_version?: string
				license: string
				client_side: Environment
				server_side: Environment
				gallery: string[]
				color?: number
			}

			export type SearchResult = {
				hits: SearchResultHit[]
				offset: number
				limit: number
				total_hits: number
			}

			export type ProjectSearchParams = {
				query?: string
				facets?: string[][] // in the format of [["categories:forge"],["versions:1.17.1"]]
				filters?: string
				index?: 'relevance' | 'downloads' | 'follows' | 'newest' | 'updated'
				offset?: number
				limit?: number
			}
		}

		export namespace v3 {
			export type Environment =
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

			export type GalleryItem = {
				url: string
				raw_url: string
				featured: boolean
				name?: string
				description?: string
				created: string
				ordering: number
			}

			export type Link = {
				platform: string
				donation: boolean
				url: string
			}

			export type Project = {
				id: string
				slug?: string
				project_types: string[]
				games: string[]
				team_id: string
				organization?: string
				name: string
				summary: string
				description: string
				published: string
				updated: string
				approved?: string
				queued?: string
				status: v2.ProjectStatus
				requested_status?: v2.ProjectStatus
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
				versions: string[]
				icon_url?: string
				link_urls: Record<string, Link>
				gallery: GalleryItem[]
				color?: number
				thread_id: string
				monetization_status: v2.MonetizationStatus
				side_types_migration_review_status: 'reviewed' | 'pending'
				environment?: Environment[]

				/**
				 * @deprecated Not recommended to use.
				 **/
				[key: string]: unknown
			}

			export type EditProjectRequest = {
				name?: string
				summary?: string
				description?: string
				categories?: string[]
				additional_categories?: string[]
				license_url?: string | null
				link_urls?: Record<string, string | null>
				license_id?: string
				slug?: string
				status?: v2.ProjectStatus
				requested_status?: v2.ProjectStatus | null
				moderation_message?: string | null
				moderation_message_body?: string | null
				monetization_status?: v2.MonetizationStatus
				side_types_migration_review_status?: 'reviewed' | 'pending'
				environment?: Environment
				[key: string]: unknown
			}
		}
	}

	export namespace Versions {
		export namespace v2 {
			export type VersionType = 'release' | 'beta' | 'alpha'

			export type VersionStatus =
				| 'listed'
				| 'archived'
				| 'draft'
				| 'unlisted'
				| 'scheduled'
				| 'unknown'

			export type DependencyType = 'required' | 'optional' | 'incompatible' | 'embedded'

			export type FileType = 'required-resource-pack' | 'optional-resource-pack' | 'unknown'

			export type VersionFile = {
				hashes: Record<string, string>
				url: string
				filename: string
				primary: boolean
				size: number
				file_type?: FileType
			}

			export type Dependency = {
				file_name?: string
				dependency_type: DependencyType
			} & (
				| {
						project_id: string
				  }
				| {
						version_id: string
						project_id?: string
				  }
			)

			export type Version = {
				id: string
				project_id: string
				author_id: string
				featured: boolean
				name: string
				version_number: string
				changelog: string
				changelog_url?: string | null
				date_published: string
				downloads: number
				version_type: VersionType
				status: VersionStatus
				requested_status?: VersionStatus | null
				files: VersionFile[]
				dependencies: Dependency[]
				game_versions: string[]
				loaders: string[]
			}
		}

		// TODO: consolidate duplicated types between v2 and v3 versions
		export namespace v3 {
			export interface Dependency {
				dependency_type: Labrinth.Versions.v2.DependencyType
				project_id?: string
				file_name?: string
				version_id?: string
			}

			export interface GetProjectVersionsParams {
				game_versions?: string[]
				loaders?: string[]
			}

			export type VersionChannel = 'release' | 'beta' | 'alpha'

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

			interface VersionFile {
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
				game_versions: string[]
				version_type: VersionChannel
				loaders: string[]
				featured: boolean
				status: Labrinth.Versions.v2.VersionStatus
				id: string
				project_id: string
				author_id: string
				date_published: string
				downloads: number
				files: VersionFile[]
				environment?: Labrinth.Projects.v3.Environment
				mrpack_loaders?: string[]
			}

			export interface DraftVersionFile {
				fileType?: FileType
				file: File
			}

			export type DraftVersion = Omit<
				Labrinth.Versions.v3.CreateVersionRequest,
				'file_parts' | 'primary_file' | 'file_types'
			> & {
				existing_files?: VersionFile[]
				version_id?: string
				environment?: Labrinth.Projects.v3.Environment
			}

			export interface CreateVersionRequest {
				name: string
				version_number: string
				changelog: string
				dependencies?: Array<{
					version_id?: string
					project_id?: string
					file_name?: string
					dependency_type: Labrinth.Versions.v2.DependencyType
				}>
				game_versions: string[]
				version_type: 'release' | 'beta' | 'alpha'
				loaders: string[]
				featured?: boolean
				status?: 'listed' | 'archived' | 'draft' | 'unlisted' | 'scheduled' | 'unknown'
				requested_status?: 'listed' | 'archived' | 'draft' | 'unlisted' | null
				project_id: string
				file_parts: string[]
				primary_file?: string
				file_types?: Record<string, Labrinth.Versions.v3.FileType | null>
				environment?: Labrinth.Projects.v3.Environment
				mrpack_loaders?: string[]
			}

			export type ModifyVersionRequest = Partial<
				Omit<CreateVersionRequest, 'project_id' | 'file_parts' | 'primary_file' | 'file_types'>
			> & {
				file_types?: {
					algorithm: string
					hash: string
					file_type: Labrinth.Versions.v3.FileType | null
				}[]
			}

			export type AddFilesToVersionRequest = {
				file_parts: string[]
				file_types?: Record<string, Labrinth.Versions.v3.FileType | null>
			}
		}
	}

	export namespace Users {
		namespace Common {
			export type Role = 'developer' | 'moderator' | 'admin'

			export type AuthProvider =
				| 'github'
				| 'discord'
				| 'microsoft'
				| 'gitlab'
				| 'google'
				| 'steam'
				| 'paypal'

			export type UserPayoutData = {
				paypal_address?: string
				paypal_country?: string
				venmo_handle?: string
				balance: number
			}
		}

		export namespace v2 {
			export type Role = Common.Role
			export type AuthProvider = Common.AuthProvider
			export type UserPayoutData = Common.UserPayoutData

			export type User = {
				id: string
				username: string
				name?: string
				avatar_url?: string
				bio?: string
				created: string
				role: Role
				badges: number
				auth_providers?: AuthProvider[]
				email?: string
				email_verified?: boolean
				has_password?: boolean
				has_totp?: boolean
				payout_data?: UserPayoutData
				github_id?: number
			}
		}

		export namespace v3 {
			export type Role = Common.Role
			export type AuthProvider = Common.AuthProvider
			export type UserPayoutData = Common.UserPayoutData

			export type User = {
				id: string
				username: string
				avatar_url?: string
				bio?: string
				created: string
				role: Role
				badges: number
				auth_providers?: AuthProvider[]
				email?: string
				email_verified?: boolean
				has_password?: boolean
				has_totp?: boolean
				payout_data?: UserPayoutData
				stripe_customer_id?: string
				allow_friend_requests?: boolean
				github_id?: number
			}
		}
	}

	export namespace Tags {
		export namespace v2 {
			export interface Category {
				icon: string
				name: string
				project_type: string
				header: string
			}

			export interface Loader {
				icon: string
				name: string
				supported_project_types: string[]
			}

			export interface GameVersion {
				version: string
				version_type: string
				date: string
				major: boolean
			}

			export interface DonationPlatform {
				short: string
				name: string
			}
		}
	}

	export namespace Search {
		export namespace v2 {
			export interface ResultSearchProject {
				project_id: string
				project_type: string
				slug: string | null
				author: string
				title: string
				description: string
				categories: string[]
				display_categories: string[]
				versions: string[]
				downloads: number
				follows: number
				icon_url: string
				date_created: string
				date_modified: string
				latest_version: string
				license: string
				client_side: string
				server_side: string
				gallery: string[]
				featured_gallery: string | null
				color: number | null
			}

			export interface SearchResults {
				hits: ResultSearchProject[]
				offset: number
				limit: number
				total_hits: number
			}
		}
	}

	export namespace Collections {
		export type CollectionStatus = 'listed' | 'unlisted' | 'private' | 'rejected' | 'unknown'

		export type Collection = {
			id: string
			user: string
			name: string
			description: string | null
			icon_url: string | null
			color: number | null
			status: CollectionStatus
			created: string
			updated: string
			projects: string[]
		}

		export type EditCollectionRequest = {
			name?: string
			description?: string | null
			status?: CollectionStatus
			new_projects?: string[]
		}
	}

	export namespace State {
		export interface GeneratedState {
			categories: Tags.v2.Category[]
			loaders: Tags.v2.Loader[]
			gameVersions: Tags.v2.GameVersion[]
			donationPlatforms: Tags.v2.DonationPlatform[]
			reportTypes: string[]
			muralBankDetails?: Record<
				string,
				{
					bankNames: string[]
				}
			>

			homePageProjects?: Projects.v2.Project[]
			homePageSearch?: Search.v2.SearchResults
			homePageNotifs?: Search.v2.SearchResults
			products?: Billing.Internal.Product[]

			countries: ISO3166.Country[]
			subdivisions: Record<string, ISO3166.Subdivision[]>

			errors: unknown[]
		}
	}

	export namespace TechReview {
		export namespace Internal {
			export type SearchProjectsRequest = {
				limit?: number
				page?: number
				filter?: SearchProjectsFilter
				sort_by?: SearchProjectsSort
			}

			export type SearchProjectsFilter = {
				project_type?: string[]
			}

			export type SearchProjectsSort =
				| 'created_asc'
				| 'created_desc'
				| 'severity_asc'
				| 'severity_desc'

			export type UpdateIssueRequest = {
				verdict: 'safe' | 'unsafe'
			}

			export type SubmitProjectRequest = {
				verdict: 'safe' | 'unsafe'
				message?: string
			}

			export type SearchResponse = {
				project_reports: ProjectReport[]
				projects: Record<string, ProjectModerationInfo>
				threads: Record<string, Thread>
				ownership: Record<string, Ownership>
			}

			export type ProjectModerationInfo = {
				id: string
				thread_id: string
				name: string
				project_types: string[]
				icon_url: string | null
			} & Projects.v3.Project

			export type ProjectReport = {
				project_id: string
				max_severity: DelphiSeverity | null
				versions: VersionReport[]
			}

			export type VersionReport = {
				version_id: string
				files: FileReport[]
			}

			export type FileReport = {
				report_id: string
				file_id: string
				created: string
				flag_reason: FlagReason
				severity: DelphiSeverity
				file_name: string
				file_size: number
				download_url: string
				issues: FileIssue[]
			}

			export type FileIssue = {
				id: string
				report_id: string
				issue_type: string
				details: ReportIssueDetail[]
			}

			export type ReportIssueDetail = {
				id: string
				issue_id: string
				key: string
				file_path: string
				decompiled_source: string | null
				data: Record<string, unknown>
				severity: DelphiSeverity
				status: DelphiReportIssueStatus
			}

			export type Ownership =
				| {
						kind: 'user'
						id: string
						name: string
						icon_url?: string
				  }
				| {
						kind: 'organization'
						id: string
						name: string
						icon_url?: string
				  }

			export type DBThread = {
				id: string
				project_id?: string
				report_id?: string
				type_: ThreadType
				messages: DBThreadMessage[]
				members: string[]
			}

			export type DBThreadMessage = {
				id: string
				thread_id: string
				author_id?: string
				body: MessageBody
				created: string
				hide_identity: boolean
			}

			export type MessageBody =
				| {
						type: 'text'
						body: string
						private?: boolean
						replying_to?: string
						associated_images?: string[]
				  }
				| {
						type: 'status_change'
						new_status: Projects.v2.ProjectStatus
						old_status: Projects.v2.ProjectStatus
				  }
				| {
						type: 'thread_closure'
				  }
				| {
						type: 'thread_reopen'
				  }
				| {
						type: 'deleted'
						private?: boolean
				  }

			export type ThreadType = 'report' | 'project' | 'direct_message'

			export type User = {
				id: string
				username: string
				avatar_url: string
				role: string
				badges: number
				created: string
				bio?: string
			}

			export type ThreadMessage = {
				id: string | null
				author_id: string | null
				body: MessageBody
				created: string
				hide_identity: boolean
			}

			export type Thread = {
				id: string
				type: ThreadType
				project_id: string | null
				report_id: string | null
				messages: ThreadMessage[]
				members: User[]
			}

			export type FlagReason = 'delphi'

			export type DelphiSeverity = 'low' | 'medium' | 'high' | 'severe'

			export type DelphiReportIssueStatus = 'pending' | 'safe' | 'unsafe'
		}
	}
}
