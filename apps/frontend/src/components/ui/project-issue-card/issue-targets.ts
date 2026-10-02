import type { Labrinth } from '@modrinth/api-client'

type Target = Labrinth.Threads.v3.ThreadIssueTarget

export function threadIssueSettingsArea(target: Target): string {
	switch (target.type) {
		case 'modify_title':
		case 'modify_slug':
		case 'modify_summary':
			return ''
		case 'modify_description':
			return 'description'
		case 'modify_license':
			return 'license'
		case 'modify_icon':
			return ''
		case 'remove_tags':
			return 'tags'
		case 'modify_links':
			return 'links'
		case 'add_gallery_images':
		case 'modify_gallery_image':
		case 'remove_gallery_images':
			return 'gallery'
		case 'remove_project_disclosures':
		case 'modify_project_disclosure':
		case 'modify_project_disclosure_note':
			return 'disclosures'
		case 'version':
			return 'versions'
		case 'modify_team_member_role':
			return 'members'
		case 'modify_server_languages':
		case 'modify_server_address':
			return 'server'
		case 'acknowledge':
			return ''
	}
}
