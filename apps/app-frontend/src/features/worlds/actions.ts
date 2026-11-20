import { get_full_path } from '@/helpers/instance'
import { openPath } from '@/helpers/utils'

export async function showWorldInFolder(instanceId: string, worldPath: string) {
	const fullPath = await get_full_path(instanceId)
	return await openPath(`${fullPath}/saves/${worldPath}`)
}
