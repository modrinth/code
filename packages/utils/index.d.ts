export const BASE62_CHARS = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'
type Base62Char = (typeof BASE62_CHARS)[number]

declare global {
  type ModrinthId = `${Base62Char}`[]

  type Environment = 'required' | 'optional' | 'unsupported' | 'unknown'

  type RequestableStatus = 'approved' | 'archived' | 'unlisted' | 'private'
  type ApprovedStatus = RequestableStatus | 'scheduled'
  type UnapprovedStatus = 'draft' | 'processing' | 'rejected' | 'withheld'
  type ProjectStatus = ApprovedStatus | UnapprovedStatus | 'unknown'

  type DonationPlatform =
    | { short: 'patreon'; name: 'Patreon' }
    | { short: 'bmac'; name: 'Buy Me A Coffee' }
    | { short: 'paypal'; name: 'PayPal' }
    | { short: 'github'; name: 'GitHub Sponsors' }
    | { short: 'ko-fi'; name: 'Ko-fi' }
    | { short: 'other'; name: 'Other' }

  type ProjectType = 'mod' | 'modpack' | 'resourcepack' | 'shader'
  type MonetizationStatus = 'monetized' | 'demonetized' | 'force-demonetized'

  type GameVersion = string
  type Platform = string
  type Category = string
  type CategoryOrPlatform = Category | Platform

  interface DonationLink<T extends DonationPlatform> {
    id: T['short']
    platform: T['name']
    url: string
  }

  interface GalleryImage {
    url: string
    featured: boolean
    created: string
    ordering: number

    title?: string
    description?: string
  }

  interface Project {
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

    team: ModrinthId
    thread_id: ModrinthId

    issues_url?: string
    source_url?: string
    wiki_url?: string
    discord_url?: string
    donation_urls: DonationLink[]

    published: string
    updated: string
    approved: string
    queued: string

    game_versions: GameVersion[]
    loaders: Platform[]

    versions: ModrinthId[]
    gallery?: GalleryImage[]

    license: {
      id: string
      name
      string
      url?: string
    }
  }

  interface SearchResult {
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

  type DependencyType = 'required' | 'optional' | 'incompatible' | 'embedded'

  interface VersionDependency {
    dependency_type: DependencyType
    file_name?: string
  }

  interface ProjectDependency {
    dependency_type: DependencyType
    project_id?: string
  }

  interface FileDependency {
    dependency_type: DependencyType
    file_name?: string
  }

  type Dependency = VersionDependency | ProjectDependency | FileDependency
  type VersionChannel = 'release' | 'beta' | 'alpha'
  type VersionStatus = 'listed' | 'archived' | 'draft' | 'unlisted' | 'scheduled' | 'unknown'
  type FileType = 'required-resource-pack' | 'optional-resource-pack'

  interface VersionFileHash {
    sha512: string
    sha1: string
  }

  interface VersionFile {
    hashes: VersionFileHash[]
    url: string
    filename: string
    primary: boolean
    size: number
    file_type?: FileType
  }

  interface Version {
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

  interface PayoutData {
    balance: number
    payout_wallet: 'paypal' | 'venmo'
    payout_wallet_type: 'email' | 'phone' | 'user_handle'
    payout_address: string
  }

  type UserRole = 'admin' | 'moderator' | 'pyro' | 'developer'

  enum UserBadge {
    MIDAS = 1 << 0,
    EARLY_MODPACK_ADOPTER = 1 << 1,
    EARLY_RESPACK_ADOPTER = 1 << 2,
    EARLY_PLUGIN_ADOPTER = 1 << 3,
    ALPHA_TESTER = 1 << 4,
    CONTRIBUTOR = 1 << 5,
    TRANSLATOR = 1 << 6,
  }

  type UserBadges = number

  interface User {
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

  enum TeamMemberPermission {
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

  type TeamMemberPermissions = number

  interface TeamMember {
    team_id: ModrinthId
    user: User
    role: string
    permissions: TeamMemberPermissions
    accepted: boolean
    payouts_split: number
    ordering: number
  }
}
