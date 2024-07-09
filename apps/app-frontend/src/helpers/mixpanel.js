import mixpanel from 'mixpanel-browser'

// mixpanel_track
function trackWrapper(originalTrack) {
  return function (event_name, properties = {}) {
    try {
      originalTrack(event_name, properties)
    } catch (e) {
      console.error(e)
    }
  }
}
export const mixpanel_track = trackWrapper(mixpanel.track.bind(mixpanel))

// mixpanel_opt_out_tracking()
function optOutTrackingWrapper(originalOptOutTracking) {
  return function () {
    try {
      originalOptOutTracking()
    } catch (e) {
      console.error(e)
    }
  }
}
export const mixpanel_opt_out_tracking = optOutTrackingWrapper(
  mixpanel.opt_out_tracking.bind(mixpanel),
)

// mixpanel_opt_in_tracking()
function optInTrackingWrapper(originalOptInTracking) {
  return function () {
    try {
      originalOptInTracking()
    } catch (e) {
      console.error(e)
    }
  }
}
export const mixpanel_opt_in_tracking = optInTrackingWrapper(
  mixpanel.opt_in_tracking.bind(mixpanel),
)

// mixpanel_init
function initWrapper(originalInit) {
  return function (token, config = {}) {
    try {
      originalInit(token, config)
    } catch (e) {
      console.error(e)
    }
  }
}
export const mixpanel_init = initWrapper(mixpanel.init.bind(mixpanel))

export const mixpanel_is_loaded = () => {
  return mixpanel.__loaded
}
