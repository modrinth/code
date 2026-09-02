import { defineMessages } from '@modrinth/ui/i18n'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { evaluateNonStandardText, evaluateProfanity, evaluateSlur } from '../text.ts'
import { toFieldMessages } from '../to-field-messages.ts'
import { toNags } from '../to-nags.ts'
import type { FieldValidationMessage, ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	uploadImage: {
		id: 'nags.upload-gallery-image.title',
		defaultMessage: `Upload a gallery image`,
	},
	uploadResourcePackImage: {
		id: 'nags.upload-gallery-image.description-resourcepack',
		defaultMessage: `At least one gallery image is required to showcase the content of your resource pack, except for audio or localization packs. If this describes your pack, please select the appropriate tag.`,
	},
	uploadShaderImages: {
		id: 'nags.upload-gallery-image.description-shader',
		defaultMessage: `At least three gallery images are required to showcase the content of your shader in a variety of situations and conditions.`,
	},
	uploadImageDescription: {
		id: 'nags.upload-gallery-image.description',
		defaultMessage: `At least one gallery image is required to showcase the content of your {type}.`,
	},
	featureImage: {
		id: 'nags.feature-gallery-image.title',
		defaultMessage: `Feature a gallery image`,
	},
	featureImageDescription: {
		id: 'nags.feature-gallery-image.description',
		defaultMessage: `The featured gallery image is often how your project makes its first impression.`,
	},
	fixText: {
		id: 'nags.invalid-gallery-text.title',
		defaultMessage: `Modify gallery image text`,
	},
	editGallery: {
		id: 'nags.edit-gallery.title',
		defaultMessage: `Edit gallery`,
	},
	slur: {
		id: 'nags.gallery-text-slur.description',
		defaultMessage: `Your gallery image titles and descriptions must not contain offensive terms. Detected: “{value}”.`,
	},
	profanity: {
		id: 'nags.gallery-text-profanity.description',
		defaultMessage: `Your gallery image titles and descriptions cannot contain excessive profanity. Detected: “{value}”.`,
	},
	nonStandardText: {
		id: 'nags.gallery-text-non-standard.description',
		defaultMessage: `Non-standard text characters, such as “Fancy text” or “Zalgo”, are not allowed in gallery image titles or descriptions.`,
	},
})

type GalleryTextInput = string | null | undefined

export const projectGalleryTextValidationRules = {
	'gallery-text-slur': {
		severity: 'error',
		evaluate: (text) => evaluateSlur(text ?? ''),
		presentation: {
			message: messages.slur,
			nag: {
				title: messages.fixText,
				destination: 'gallery',
				linkTitle: messages.editGallery,
			},
		},
	},
	'gallery-text-profanity': {
		severity: 'error',
		evaluate: (text) => evaluateProfanity(text ?? ''),
		presentation: {
			message: messages.profanity,
			nag: {
				title: messages.fixText,
				destination: 'gallery',
				linkTitle: messages.editGallery,
			},
		},
	},
	'gallery-text-non-standard': {
		severity: 'error',
		evaluate: (text) => evaluateNonStandardText(text ?? ''),
		presentation: {
			message: messages.nonStandardText,
			nag: {
				title: messages.fixText,
				destination: 'gallery',
				linkTitle: messages.editGallery,
			},
		},
	},
} satisfies ValidationRuleSet<GalleryTextInput>

export const projectGalleryValidationRules = {
	'upload-gallery-image': {
		severity: 'error',
		evaluate: (context) => {
			const isShader = context.projectV3.project_types.includes('shader')
			if (isShader && context.project.gallery && context.project.gallery.length < 3) {
				return { valid: false, message: messages.uploadShaderImages }
			}

			const isResourcePack = context.projectV3.project_types.includes('resourcepack')
			const categories = context.project.categories.concat(
				context.project.additional_categories ?? [],
			)
			if (
				isResourcePack &&
				context.project.gallery &&
				context.project.gallery.length === 0 &&
				!categories.includes('audio') &&
				!categories.includes('locale')
			) {
				return { valid: false, message: messages.uploadResourcePackImage }
			}

			return { valid: true }
		},
		presentation: {
			message: messages.uploadImageDescription,
			nag: { title: messages.uploadImage, destination: 'gallery' },
		},
	},
	'feature-gallery-image': {
		severity: 'suggestion',
		evaluate: (context) => ({
			valid:
				Boolean(context.projectV3.minecraft_server) ||
				Boolean(context.project.gallery?.find((image) => image.featured)),
		}),
		presentation: {
			message: messages.featureImageDescription,
			nag: { title: messages.featureImage, destination: 'gallery' },
		},
	},
} satisfies ValidationRuleSet<ProjectValidationContext>

export function validateProjectGalleryName(name: GalleryTextInput): FieldValidationMessage[] {
	return toFieldMessages(evaluateRules(name, projectGalleryTextValidationRules))
}

export function validateProjectGalleryDescription(
	description: GalleryTextInput,
): FieldValidationMessage[] {
	return toFieldMessages(evaluateRules(description, projectGalleryTextValidationRules))
}

export function getGalleryNags(context: ProjectValidationContext): Nag[] {
	const galleryNags = toNags(evaluateRules(context, projectGalleryValidationRules))
	const textNags = context.projectV3.gallery.flatMap((item, index) => {
		const nameNags = toNags(evaluateRules(item.name, projectGalleryTextValidationRules)).map(
			(nag) => ({ ...nag, id: `${nag.id}-${index}-name` }),
		)
		const descriptionNags = toNags(
			evaluateRules(item.description, projectGalleryTextValidationRules),
		).map((nag) => ({ ...nag, id: `${nag.id}-${index}-description` }))

		return [...nameNags, ...descriptionNags]
	})

	return [...galleryNags, ...textNags]
}
