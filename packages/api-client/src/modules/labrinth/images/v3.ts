import { AbstractModule } from '../../../core/abstract-module.js'
import type { UploadHandle } from '../../../types/upload.js'
import type { Labrinth } from '../types.js'

function buildImageQueryParams(
	ext: Labrinth.Images.v3.ImageExtension,
	target: Labrinth.Images.v3.UploadImageParams,
): Record<string, string> {
	const params: Record<string, string> = {
		ext,
		context: target.context,
	}
	switch (target.context) {
		case 'project':
			params.project_id = target.project_id
			break
		case 'version':
			params.version_id = target.version_id
			break
		case 'thread_message':
			params.thread_message_id = target.thread_message_id
			break
		case 'report':
			params.report_id = target.report_id
			break
	}
	return params
}

export class LabrinthImagesV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_images_v3'
	}

	public uploadImage(
		file: File | Blob,
		ext: Labrinth.Images.v3.ImageExtension,
		target: Labrinth.Images.v3.UploadImageParams,
	): UploadHandle<Labrinth.Images.v3.UploadedImage> {
		return this.client.upload<Labrinth.Images.v3.UploadedImage>('/image', {
			api: 'labrinth',
			version: 3,
			file,
			params: buildImageQueryParams(ext, target),
		})
	}
}
