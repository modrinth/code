import ace from 'ace-builds'

// Ace's resolver and language bundles reference a global, even when loaded as modules.
;(globalThis as typeof globalThis & { ace: typeof ace }).ace = ace

export default ace
