export type ReviewTarget =
	| {
			kind:
				| 'title'
				| 'slug'
				| 'icon'
				| 'summary'
				| 'license'
				| 'license-url'
				| 'description'
				| 'gallery'
				| 'disclosures'
				| 'versions'
	  }
	| { kind: 'link' | 'gallery-image' | 'version'; key: string }
