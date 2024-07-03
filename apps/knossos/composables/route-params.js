/**
 * Extracts the [id] from the route params and returns it as a ref.
 *
 * @param {string?} key The key of the route param to extract.
 * @returns {import('vue').Ref<string | string[] | undefined>}
 */
export const useRouteId = (key = 'id') => {
  const route = useNativeRoute()
  return route.params?.[key] || undefined
}
