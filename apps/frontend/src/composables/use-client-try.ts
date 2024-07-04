type AsyncFunction<TArgs extends any[], TResult> = (...args: TArgs) => Promise<TResult>
type ErrorFunction = (err: any) => void | Promise<void>
type VoidFunction = () => void | Promise<void>

type useClientTry = <TArgs extends any[], TResult>(
  fn: AsyncFunction<TArgs, TResult>,
  onFail?: ErrorFunction,
  onFinish?: VoidFunction
) => (...args: TArgs) => Promise<TResult | undefined>

const defaultOnError: ErrorFunction = (error) => {
  addNotification({
    group: 'main',
    title: 'An error occurred',
    text: error?.data?.description || error.message || error || 'Unknown error',
    type: 'error',
  })
}

export const useClientTry: useClientTry =
  (fn, onFail = defaultOnError, onFinish) =>
  async (...args) => {
    startLoading()
    try {
      return await fn(...args)
    } catch (err) {
      if (onFail) {
        await onFail(err)
      } else {
        console.error('[CLIENT TRY ERROR]', err)
      }
    } finally {
      if (onFinish) await onFinish()
      stopLoading()
    }
  }
