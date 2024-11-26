type ImageUploadContext = {
  projectID?: string;
  context: 'project' | 'version' | 'thread_message' | 'report';
};

interface ImageUploadResponse {
  id: string;
  url: string;
}

export const useImageUpload = async (file: File, ctx: ImageUploadContext) => {
  // 确保文件类型为 image/png、image/jpeg、image/gif 或 image/webp
  if (
    !file.type.startsWith('image/') ||
    !['png', 'jpeg', 'gif', 'webp'].includes(file.type.split('/')[1])
  ) {
    throw new Error('文件不是接受的图片类型,仅接受png、jpeg、gif或webp')
  }

// 确保文件小于 1MB
  if (file.size > 1024 * 1024) {
    throw new Error('文件太大，请小于1MB')
  }

  const qs = new URLSearchParams()
  if (ctx.projectID) qs.set('project_id', ctx.projectID)
  qs.set('context', ctx.context)
  qs.set('ext', file.type.split('/')[1])
  const url = `image?${qs.toString()}`

  const response = (await useBaseFetch(url, {
    method: 'POST',
    body: file,
    apiVersion: 3
  })) as ImageUploadResponse

// 类型检查以查看响应是否具有 url 属性和 id 属性
  if (!response?.id || typeof response.id !== 'string') {
    throw new Error('服务器响应异常')
  }
  if (!response?.url || typeof response.url !== 'string') {
    throw new Error('服务器响应异常')
  }

  return response
}
