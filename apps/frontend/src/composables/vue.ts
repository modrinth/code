/**
 * Creates a computed reference that uses a provide getter function called with an argument representing the current mount state of the component.
 * @param getter A getter function that will run with `mounted` argument representing whether or not the component is mounted.
 * @returns A computed reference that changes when component becomes mounted or unmounted.
 */
export function useMountedValue<T>(getter: (isMounted: boolean) => T) {
  const mounted = ref(getCurrentInstance()?.isMounted ?? false);

  onMounted(() => (mounted.value = true));

  onUnmounted(() => (mounted.value = false));

  return computed(() => getter(mounted.value));
}
