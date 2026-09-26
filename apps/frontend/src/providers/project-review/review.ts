export type ReviewTarget =
	| {
			kind:
				| 'title'
				| 'slug'
				| 'icon'
				| 'summary'
				| 'tags'
				| 'compatibility'
				| 'server'
				| 'reupload'
				| 're-review'
				| 'post-approval'
				| 'status-alerts'
				| 'undefined-project'
				| 'license'
				| 'license-url'
				| 'description'
				| 'gallery'
				| 'disclosures'
				| 'versions'
				| 'permissions'
				| 'rules'
	  }
	| { kind: 'link' | 'gallery-image' | 'version' | 'disclosure'; key: string }
