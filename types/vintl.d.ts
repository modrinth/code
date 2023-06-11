import '@vintl/vintl'
import { CompactNumber } from '@vintl/compact-number/dist/index.mjs'

declare global {
  namespace VueIntlController {
    interface MessageValueTypes {
      compactNumber: CompactNumber
    }
  }
}
