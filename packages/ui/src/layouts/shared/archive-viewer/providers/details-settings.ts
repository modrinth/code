import { useLocalStorage } from '@vueuse/core'
import type { Ref } from 'vue'

import type { HashType } from '#ui/helpers/toolbox/file'
import { createContext } from '#ui/providers/create-context'

/** When a kind of detail shows: always, only with an item's details expanded, or never. */
export type DetailVisibility = 'always' | 'details' | 'never'

/** What the archive viewer's details show, which hosts can tie to settings of their own. */
export interface ArchiveDetailsSettings {
	/** Hashes listed in an item's details, which are only computed once shown. */
	hashTypes: Ref<HashType[]>
	/** When an archive's counts of files, folders, class files and nested archives show. */
	contentCounts: Ref<DetailVisibility>
}

export const DEFAULT_ARCHIVE_HASH_TYPES: HashType[] = ['sha1', 'sha512', 'murmur2', 'crc32']

export const [injectArchiveDetailsSettings, provideArchiveDetailsSettings] =
	createContext<ArchiveDetailsSettings>('ArchiveViewerLayout', 'archiveDetailsSettings')

/** Settings kept in local storage, for hosts without settings of their own. */
export function useStoredArchiveDetailsSettings(): ArchiveDetailsSettings {
	return {
		hashTypes: useLocalStorage<HashType[]>(
			'archive-viewer-details:hash-types',
			DEFAULT_ARCHIVE_HASH_TYPES,
		),
		contentCounts: useLocalStorage<DetailVisibility>(
			'archive-viewer-details:content-counts',
			'details',
		),
	}
}
