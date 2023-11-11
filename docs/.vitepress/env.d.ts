/// <reference types="vite/client" />

declare module '@modrinth/omorphia-dev/locales/index.js' {
  interface LocaleExport {
    messages: Record<string, any[]>
  }

  interface LocaleDefinition {
    importFunction(): Promise<LocaleExport>
  }

  const localeDefinitions: Partial<Record<string, LocaleDefinition>>

  export { localeDefinitions }
}
