import { posthog } from 'posthog-js'

export const initAnalytics = () => {
  posthog.init('phc_9Iqi6lFs9sr5BSqh9RRNRSJ0mATS9PSgirDiX3iOYJ', {
    persistence: 'localStorage',
    api_host: 'https://posthog.modrinth.com',
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
