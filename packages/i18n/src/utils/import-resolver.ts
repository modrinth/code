import { match as matchLocale, type Opts as MatchLocaleOpts } from '@formatjs/intl-localematcher'
import {
  normalizeImportSource,
  normalizeMessagesImportSource,
  normalizeUnspecifiedImportSource,
  type ImportSource,
  type ImportSourceObject,
  type MessagesImportSource,
  type MessagesImportSourceObject,
  type UnspecifiedImportSource,
  type UnspecifiedImportSourceObject,
} from '@vintl/nuxt/options'

export interface CollectorContext {
  addFile(locale: string, file: MessagesImportSource): void
  addResource(locale: string, file: ImportSource): void
  addImport(locale: string, import_: UnspecifiedImportSource): void
}

export interface ResolvableEntry {
  files: MessagesImportSourceObject[]
  resources: ImportSourceObject[]
  imports: UnspecifiedImportSourceObject[]
}

export type Resolver = (locale: string) => ResolvableEntry | undefined

export async function createLocaleResolver(
  collector: (ctx: CollectorContext) => void | Promise<void>,
): Promise<Resolver> {
  const entries = new Map<string, ResolvableEntry>()

  function getEntry(locale: string) {
    let entry = entries.get(locale)
    if (entry == null) {
      entry = { files: [], resources: [], imports: [] }
      entries.set(locale, entry)
    }
    return entry
  }

  const ctx: CollectorContext = {
    addFile(locale, file) {
      getEntry(locale).files.push(normalizeMessagesImportSource(file))
    },
    addResource(locale, file) {
      getEntry(locale).resources.push(normalizeImportSource(file))
    },
    addImport(locale, import_) {
      getEntry(locale).imports.push(normalizeUnspecifiedImportSource(import_))
    },
  }

  await collector(ctx)

  const availableLocales = [...entries.keys()]

  return function resolveLocale(locales: string | string[], opts?: MatchLocaleOpts) {
    const requestedLocales = Array.isArray(locales) ? locales : [locales]
    const match = matchLocale(requestedLocales, availableLocales, 'en-x-placeholder', opts)
    if (match === 'en-x-placeholder') return
    return entries.get(match)!
  }
}
