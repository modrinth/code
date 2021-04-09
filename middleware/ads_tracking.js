/* eslint-disable no-undef */
export default function ({ route }) {
  if (process.client) {
    googletag.cmd.push(function () {
      googletag.pubads().setTargeting('path', route.path)
    })
  }
}
