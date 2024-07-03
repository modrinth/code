export type AutoRef<T> = [T] extends [(...args: any[]) => any]
  ? Ref<T> | (() => T)
  : T | Ref<T> | (() => T)

/**
 * Accepts a value directly, a ref with the value or a getter function and returns a Vue ref.
 * @param value The value to use.
 * @returns Either the original or newly created ref.
 */
export function useAutoRef<T>(value: AutoRef<T>): Ref<T> {
  if (typeof value === 'function') return computed(() => value())
  return isRef(value) ? value : ref(value as any)
}
