import { posthog } from 'posthog-js'

export const initAnalytics = () => {
  posthog.init('phc_hm2ihMpTAoE86xIm7XzsCB8RPiTRKivViK5biiHedm', {
    persistence: 'localStorage',
  })
}

export const debugAnalytics = () => {
  posthog.debug()
}

export const optOutAnalytics = () => {
  posthog.opt_out_capturing()
}

export const optInAnalytics = () => {
  posthog.opt_in_capturing()
}

export const trackEvent = (eventName, properties) => {
  posthog.capture(eventName, properties)
}
