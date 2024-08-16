export {}
declare global {
  const defineMessage: (typeof import('./src/macros/define-message.ts'))['defineMessage']
}
