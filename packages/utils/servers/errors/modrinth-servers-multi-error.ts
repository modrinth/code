export class ModrinthServersMultiError extends Error {
  public readonly errors: Map<string, Error> = new Map()
  public readonly timestamp: number = Date.now()

  constructor(message?: string) {
    super(message || 'Multiple errors occurred')
    this.name = 'MultipleErrors'
  }

  addError(module: string, error: Error) {
    this.errors.set(module, error)
    this.message = this.buildErrorMessage()
  }

  hasErrors() {
    return this.errors.size > 0
  }

  private buildErrorMessage(): string {
    return (
      Array.from(this.errors.entries())
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
        .map(([_module, error]) => error.message)
        .join('\n')
    )
  }
}
