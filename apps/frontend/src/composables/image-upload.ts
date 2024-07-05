type ImageUploadContext = {
  projectID?: string
  context: 'project' | 'version' | 'thread_message' | 'report'
}

interface ImageUploadResponse {
  id: string
  url: string
}

export const useImageUpload = async (file: File, ctx: ImageUploadContext) => {
  // Make sure file is of type image/png, image/jpeg, image/gif, or image/webp
  if (
    !file.type.startsWith('image/') ||
    !['png', 'jpeg', 'gif', 'webp'].includes(file.type.split('/')[1])
  ) {
    throw new Error('File is not an accepted image type')
  }

  // Make sure file is less than 1MB
  if (file.size > 1024 * 1024) {
    throw new Error('File is too large')
  }

  const qs = new URLSearchParams()
  if (ctx.projectID) qs.set('project_id', ctx.projectID)
  qs.set('context', ctx.context)
  qs.set('ext', file.type.split('/')[1])
  const url = `image?${qs.toString()}`

  const response = (await useBaseFetch(url, {
    method: 'POST',
    body: file,
    apiVersion: 3,
  })) as ImageUploadResponse

  // Type check to see if response has a url property and an id property
  if (!response?.id || typeof response.id !== 'string') {
    throw new Error('Unexpected response from server')
  }
  if (!response?.url || typeof response.url !== 'string') {
    throw new Error('Unexpected response from server')
  }

  return response
}
