import { defineMessages } from '@modrinth/ui'

export const messages = defineMessages({
	uploadTitle: {
		id: 'project.settings.gallery.upload-title',
		defaultMessage: 'Upload gallery image',
	},
	editTitle: { id: 'project.settings.gallery.edit-title', defaultMessage: 'Edit gallery item' },
	currentImage: { id: 'project.settings.gallery.current-image', defaultMessage: 'Current image' },
	imageAlt: { id: 'project.settings.gallery.image-alt', defaultMessage: 'gallery-image' },
	toggleZoom: { id: 'project.settings.gallery.toggle-zoom', defaultMessage: 'Toggle zoom' },
	previousImage: {
		id: 'project.settings.gallery.previous-image',
		defaultMessage: 'Previous image',
	},
	nextImage: { id: 'project.settings.gallery.next-image', defaultMessage: 'Next image' },
	replace: { id: 'project.settings.gallery.replace', defaultMessage: 'Replace' },
	upload: { id: 'project.settings.gallery.upload', defaultMessage: 'Upload an image' },
	replaceImage: { id: 'project.settings.gallery.replace-image', defaultMessage: 'Replace image' },
	dateCreated: { id: 'project.settings.gallery.date-created', defaultMessage: 'Date created' },
	previewAlt: { id: 'project.settings.gallery.preview-alt', defaultMessage: 'gallery-preview' },
	titlePlaceholder: {
		id: 'project.settings.gallery.title-placeholder',
		defaultMessage: 'Enter title...',
	},
	descriptionPlaceholder: {
		id: 'project.settings.gallery.description-placeholder',
		defaultMessage: 'Enter description...',
	},
	orderPlaceholder: {
		id: 'project.settings.gallery.order-placeholder',
		defaultMessage: 'Enter order index...',
	},
	deleteTitle: {
		id: 'project.settings.gallery.delete-title',
		defaultMessage: 'Are you sure you want to delete this gallery image?',
	},
	deleteDescription: {
		id: 'project.settings.gallery.delete-description',
		defaultMessage: 'This will remove this gallery image forever (like really forever).',
	},
	title: { id: 'project.settings.gallery.title', defaultMessage: 'Title' },
	description: { id: 'project.settings.gallery.description', defaultMessage: 'Description' },
	order: { id: 'project.settings.gallery.order', defaultMessage: 'Order Index' },
	banner: { id: 'project.settings.gallery.banner', defaultMessage: 'Banner image' },
	bannerDescription: {
		id: 'project.settings.gallery.banner-description',
		defaultMessage: 'You can feature one image on your project to be used as a banner image.',
	},
	setBanner: { id: 'project.settings.gallery.set-banner', defaultMessage: 'Set as banner' },
	unsetBanner: { id: 'project.settings.gallery.unset-banner', defaultMessage: 'Unset as banner' },
	addImage: { id: 'project.settings.gallery.add-image', defaultMessage: 'Add gallery image' },
	uploadHint: {
		id: 'project.settings.gallery.upload-hint',
		defaultMessage: 'Click to choose an image or drag one onto this page',
	},
})
