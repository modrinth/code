import type { Labrinth } from '@modrinth/api-client'

export type CopyrightProjectArchive = Labrinth.CopyrightArchives.Internal.Metadata & {
	id: string
	files?: Labrinth.CopyrightArchives.Internal.Create['files']
}

export type ArchiveInput = Labrinth.CopyrightArchives.Internal.Create & { id?: string }
export type ArchiveDraft = Omit<CopyrightProjectArchive, 'minecraft_metadata'> & {
	minecraftJson: string
}

export function toDraft(archive: CopyrightProjectArchive): ArchiveDraft {
	const { minecraft_metadata, ...rest } = archive
	return {
		...rest,
		minecraftJson: minecraft_metadata ? JSON.stringify(minecraft_metadata, null, '\t') : '',
	}
}

export function fromDraft(draft: ArchiveDraft): CopyrightProjectArchive {
	const { minecraftJson, ...archive } = draft
	return {
		...archive,
		...(minecraftJson.trim() ? { minecraft_metadata: JSON.parse(minecraftJson) } : {}),
	}
}
