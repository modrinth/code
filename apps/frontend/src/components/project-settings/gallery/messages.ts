import { defineMessages } from '@modrinth/ui'

export const messages = defineMessages({
	uploadTitle: {
		id: 'project.settings.gallery.upload-title',
		defaultMessage: 'Add to gallery',
	},
	uploadProgress: {
		id: 'project.settings.gallery.upload-progress',
		defaultMessage: '{current, number} of {total, number}',
	},
	skipImage: {
		id: 'project.settings.gallery.skip-image',
		defaultMessage: 'Skip image',
	},
	invalidFileTitle: {
		id: 'project.settings.gallery.invalid-file-title',
		defaultMessage: 'Unsupported image type',
	},
	invalidFileDescription: {
		id: 'project.settings.gallery.invalid-file-description',
		defaultMessage: '{filename} must be a PNG, JPEG, GIF, or WebP image.',
	},
	editTitle: { id: 'project.settings.gallery.edit-title', defaultMessage: 'Edit gallery item' },
	galleryHelp: {
		id: 'project.settings.gallery.discovery-image-help',
		defaultMessage:
			'Upload screenshots from your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}}. Setting a featured image determines what image is shown in the Discovery card. AI-generated images are not allowed.',
	},
	emptyDescription: {
		id: 'project.settings.gallery.empty-description',
		defaultMessage: 'Upload your first gallery image.',
	},
	emptyHeading: {
		id: 'project.settings.gallery.empty-heading',
		defaultMessage: 'No gallery images yet',
	},
	upload: { id: 'project.settings.gallery.upload', defaultMessage: 'Upload images' },
	deleteTitle: {
		id: 'project.settings.gallery.delete-title',
		defaultMessage: 'Are you sure you want to delete this gallery image?',
	},
	deleteDescription: {
		id: 'project.settings.gallery.delete-description',
		defaultMessage: 'This will remove this gallery image forever (like really forever).',
	},
})
