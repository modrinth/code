export interface CopyrightProjectArchive {
	id: string
	name: string
	modrinthMetadata?: {
		projectId: string
		versionId: string
		projectType: string
	}
	minecraftMetadata?: Record<string, unknown>
	license: string
	licenseTerms?: {
		requiresSource?: string
		requiredRedistributionLicense?: string[]
		redistributionAllowed?: boolean
		commercialUseAllowed?: boolean
		attributionRequired?: boolean
	}
	files: Array<{
		path: string
		filename: string
		hash_sha512: string
	}>
}

export type ArchiveInput = Omit<CopyrightProjectArchive, 'id'> & { id?: string }
export type ArchiveDraft = Omit<CopyrightProjectArchive, 'minecraftMetadata'> & {
	minecraftJson: string
}

export function toDraft(archive: CopyrightProjectArchive): ArchiveDraft {
	const { minecraftMetadata, ...rest } = archive
	return {
		...rest,
		minecraftJson: minecraftMetadata ? JSON.stringify(minecraftMetadata, null, '\t') : '',
	}
}

export function fromDraft(draft: ArchiveDraft): CopyrightProjectArchive {
	const { minecraftJson, ...archive } = draft
	return {
		...archive,
		...(minecraftJson.trim() ? { minecraftMetadata: JSON.parse(minecraftJson) } : {}),
	}
}
