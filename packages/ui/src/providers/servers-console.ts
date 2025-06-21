import { createContext } from '.'
import type { Ref } from 'vue'

export abstract class AbstractModrinthServersConsole {
  protected readonly MAX_LINES = 10000
  protected readonly BATCH_TIMEOUT = 300 // ms
  protected readonly INITIAL_BATCH_SIZE = 256
  protected readonly REFILTER_TIMEOUT = 100 // ms

  protected lineBuffer: string[] = []
  protected batchTimer: NodeJS.Timeout | null = null
  protected refilterTimer: NodeJS.Timeout | null = null
  protected isProcessingInitialBatch = false
  protected searchRegex: RegExp | null = null

  // Direct ref properties - no need for abstract methods
  abstract readonly output: Ref<string[]>
  abstract readonly searchQuery: Ref<string>
  abstract readonly filteredOutput: Ref<string[]>

  addLine(line: string): void {
    this.lineBuffer.push(line)

    if (!this.batchTimer) {
      this.batchTimer = setTimeout(() => this.flushBuffer(), this.BATCH_TIMEOUT)
    }
  }

  addLines(lines: string[]): void {
    if (this.output.value.length === 0 && lines.length >= this.INITIAL_BATCH_SIZE) {
      this.isProcessingInitialBatch = true
      this.lineBuffer = lines
      this.flushBuffer()
      return
    }

    this.lineBuffer.push(...lines)

    if (!this.batchTimer) {
      this.batchTimer = setTimeout(() => this.flushBuffer(), this.BATCH_TIMEOUT)
    }
  }

  setSearchQuery(query: string): void {
    this.searchQuery.value = query
    this.searchRegex = null
    this.updateFilter()
  }

  clear(): void {
    this.output.value = []
    this.searchQuery.value = ""
    this.filteredOutput.value = []
    this.lineBuffer = []
    this.isProcessingInitialBatch = false
    this.searchRegex = null

    if (this.batchTimer) {
      clearTimeout(this.batchTimer)
      this.batchTimer = null
    }

    if (this.refilterTimer) {
      clearTimeout(this.refilterTimer)
      this.refilterTimer = null
    }
  }

  findLineIndex(line: string): number {
    return this.output.value.findIndex((l) => l === line)
  }

  protected flushBuffer(): void {
    if (this.lineBuffer.length === 0) return

    const processedLines = this.lineBuffer.flatMap((line) => 
      line.split('\n').filter(Boolean)
    )

    if (this.isProcessingInitialBatch && processedLines.length >= this.INITIAL_BATCH_SIZE) {
      this.isProcessingInitialBatch = false
      this.output.value = processedLines.slice(-this.MAX_LINES)
    } else {
      const newOutput = [...this.output.value, ...processedLines]
      this.output.value = newOutput.slice(-this.MAX_LINES)
    }

    this.lineBuffer = []
    this.batchTimer = null

    if (this.searchQuery.value) {
      this.scheduleRefilter()
    }
  }

  protected updateFilter(): void {
    if (!this.searchQuery.value) {
      this.filteredOutput.value = []
      return
    }

    if (!this.searchRegex) {
      this.searchRegex = new RegExp(
        this.searchQuery.value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 
        'i'
      )
    }

    this.filteredOutput.value = this.output.value.filter((line) => 
      this.searchRegex?.test(line) ?? false
    )
  }

  protected scheduleRefilter(): void {
    if (this.refilterTimer) clearTimeout(this.refilterTimer)
    this.refilterTimer = setTimeout(() => this.updateFilter(), this.REFILTER_TIMEOUT)
  }
}

export const [injectModrinthServersConsole, provideModrinthServersConsole] =
  createContext<AbstractModrinthServersConsole>('root', 'modrinthServersConsole')