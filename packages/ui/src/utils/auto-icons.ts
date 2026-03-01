import {
	ArchiveIcon,
	BoxIcon,
	BracesIcon,
	CalendarIcon,
	CardIcon,
	CurrencyIcon,
	FileArchiveIcon,
	FileCodeIcon,
	FileIcon,
	FileImageIcon,
	FileTextIcon,
	FolderOpenIcon,
	DiscordIcon,
	GithubIcon,
	GlassesIcon,
	GlobeIcon,
	InfoIcon,
	IssuesIcon,
	LinkIcon,
	LockIcon,
	PackageOpenIcon,
	PaintbrushIcon,
	PayPalIcon,
	PlugIcon,
	PolygonIcon,
	UnknownIcon,
	UpdatedIcon,
	USDCColorIcon,
	XCircleIcon,
	XIcon,
} from '@modrinth/assets'
import type { ProjectStatus, ProjectType } from '@modrinth/utils'
import type { Component } from 'vue'

import {
	FILE_ARCHIVE_EXTENSIONS,
	FILE_CODE_EXTENSIONS,
	FILE_IMAGE_EXTENSIONS,
	FILE_TEXT_EXTENSIONS,
} from './file-extensions'

export const PROJECT_TYPE_ICONS: Record<ProjectType, Component> = {
	mod: BoxIcon,
	modpack: PackageOpenIcon,
	resourcepack: PaintbrushIcon,
	shader: GlassesIcon,
	plugin: PlugIcon,
	datapack: BracesIcon,
	project: BoxIcon,
}

export const PAYMENT_METHOD_ICONS: Record<string, Component> = {
	card: CardIcon,
	cashapp: CurrencyIcon,
	paypal: PayPalIcon,
}

export const SOCIAL_PLATFORM_ICONS: Record<string, Component> = {
	discord: DiscordIcon,
	github: GithubIcon,
}

export const SEVERITY_ICONS: Record<string, Component> = {
	info: InfoIcon,
	warning: IssuesIcon,
	error: XCircleIcon,
	critical: XCircleIcon,
}

export const PROJECT_STATUS_ICONS: Record<ProjectStatus, Component> = {
	approved: GlobeIcon,
	unlisted: LinkIcon,
	withheld: LinkIcon,
	private: LockIcon,
	scheduled: CalendarIcon,
	draft: FileTextIcon,
	archived: ArchiveIcon,
	rejected: XIcon,
	processing: UpdatedIcon,
	unknown: UnknownIcon,
}

export const DIRECTORY_ICONS: Record<string, Component> = {
	config: FolderOpenIcon,
	world: FolderOpenIcon,
	resourcepacks: PaintbrushIcon,
	_default: FolderOpenIcon,
}

const CURRENCY_CONFIG: Record<string, { icon: Component; color: string }> = {
	usdc: { icon: USDCColorIcon, color: 'text-blue' },
}

const BLOCKCHAIN_CONFIG: Record<string, { icon: Component; color: string }> = {
	polygon: { icon: PolygonIcon, color: 'text-purple' },
}

export function getProjectTypeIcon(projectType: ProjectType): Component {
	return PROJECT_TYPE_ICONS[projectType] ?? BoxIcon
}

export function getPaymentMethodIcon(method: string): Component {
	return PAYMENT_METHOD_ICONS[method] ?? UnknownIcon
}

export function getSocialPlatformIcon(platform: string): Component {
	return SOCIAL_PLATFORM_ICONS[platform.toLowerCase()] ?? UnknownIcon
}

export function getSeverityIcon(severity: string): Component {
	return SEVERITY_ICONS[severity] ?? InfoIcon
}

export function getProjectStatusIcon(status: ProjectStatus): Component {
	return PROJECT_STATUS_ICONS[status] ?? UnknownIcon
}

export function getDirectoryIcon(name: string): Component {
	return DIRECTORY_ICONS[name.toLowerCase()] ?? DIRECTORY_ICONS._default
}

export function getFileExtensionIcon(extension: string): Component {
	const ext: string = extension.toLowerCase()

	if ((FILE_CODE_EXTENSIONS as readonly string[]).includes(ext)) {
		return FileCodeIcon
	}
	if ((FILE_TEXT_EXTENSIONS as readonly string[]).includes(ext)) {
		return FileTextIcon
	}
	if ((FILE_IMAGE_EXTENSIONS as readonly string[]).includes(ext)) {
		return FileImageIcon
	}
	if ((FILE_ARCHIVE_EXTENSIONS as readonly string[]).includes(ext)) {
		return FileArchiveIcon
	}

	return FileIcon
}

export function getFileIcon(fileName: string): Component {
	const extension = fileName.split('.').pop()?.toLowerCase() || ''
	return getFileExtensionIcon(extension)
}

export function getCurrencyIcon(currency: string): Component | null {
	const lower = currency.toLowerCase()
	const key = Object.keys(CURRENCY_CONFIG).find((k) => lower.includes(k))
	return key ? CURRENCY_CONFIG[key].icon : null
}

export function getCurrencyColor(currency: string): string {
	const lower = currency.toLowerCase()
	const key = Object.keys(CURRENCY_CONFIG).find((k) => lower.includes(k))
	return key ? CURRENCY_CONFIG[key].color : 'text-contrast'
}

export function getBlockchainIcon(blockchain: string): Component | null {
	const lower = blockchain.toLowerCase()
	const key = Object.keys(BLOCKCHAIN_CONFIG).find((k) => lower.includes(k))
	return key ? BLOCKCHAIN_CONFIG[key].icon : null
}

export function getBlockchainColor(blockchain: string): string {
	const lower = blockchain.toLowerCase()
	const key = Object.keys(BLOCKCHAIN_CONFIG).find((k) => lower.includes(k))
	return key ? BLOCKCHAIN_CONFIG[key].color : 'text-contrast'
}
