/** Skin previews used to be cached in IndexedDB. app-lib caches them on disk now. */
export function deleteLegacySkinPreviewStorage() {
	for (const name of ['skin-previews', 'head-storage']) {
		try {
			indexedDB.deleteDatabase(name)
		} catch (error) {
			console.warn('Could not delete legacy skin preview storage', error)
		}
	}
}
