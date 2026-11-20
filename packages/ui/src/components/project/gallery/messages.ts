import { defineMessages } from '../../../composables/i18n'

export const galleryCardMessages = defineMessages({
	imageAlt: { id: 'project.settings.gallery.image-alt', defaultMessage: 'gallery-image' },
	dateCreated: { id: 'project.settings.gallery.date-created', defaultMessage: 'Date created' },
})

export const galleryEditorMessages = defineMessages({
	replace: { id: 'project.settings.gallery.replace', defaultMessage: 'Replace' },
	replaceImage: { id: 'project.settings.gallery.replace-image', defaultMessage: 'Replace image' },
	previewAlt: { id: 'project.settings.gallery.preview-alt', defaultMessage: 'gallery-preview' },
	titlePlaceholder: {
		id: 'project.settings.gallery.title-placeholder',
		defaultMessage: 'Enter title...',
	},
	descriptionPlaceholder: {
		id: 'project.settings.gallery.description-placeholder',
		defaultMessage: 'Enter description...',
	},
	imageTitle: { id: 'project.settings.gallery.image-title', defaultMessage: 'Title' },
	description: { id: 'project.settings.gallery.description', defaultMessage: 'Description' },
	banner: { id: 'project.settings.gallery.banner', defaultMessage: 'Banner image' },
	bannerDescription: {
		id: 'project.settings.gallery.banner-description',
		defaultMessage: 'You can feature one image on your project to be used as a banner image.',
	},
	setBanner: { id: 'project.settings.gallery.set-banner', defaultMessage: 'Set as banner' },
	unsetBanner: { id: 'project.settings.gallery.unset-banner', defaultMessage: 'Unset as banner' },
})
