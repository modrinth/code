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

export type ProjectType = 'mod' | 'modpack' | 'resourcepack' | 'shader'
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

  team: ModrinthId
  thread_id: ModrinthId
  organization: ModrinthId

  issues_url?: string
  source_url?: string
  wiki_url?: string
  discord_url?: string
  donation_urls: DonationLink<DonationPlatform>[]

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
    name: string
    url?: string
  }
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
export type FileType = 'required-resource-pack' | 'optional-resource-pack'

export interface VersionFileHash {
  sha512: string
  sha1: string
}

export interface VersionFile {
  hashes: VersionFileHash[]
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

export type ServerNotice = {
  id: number
  message: string
  title?: string
  level: 'info' | 'warn' | 'critical' | 'survey'
  dismissable: boolean
  announce_at: string
  expires: string
  assigned: {
    kind: 'server' | 'node'
    id: string
    name: string
  }[]
  dismissed_by: {
    server: string
    dismissed_on: string
  }[]
}
