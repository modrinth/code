export default {
  settings: {
    ads: {
      title: 'Allow personalized ads',
      description: `Marketing/target cookies are usually used to show you advertisements that meet your interests.
       When you visit another website, your browser's cookie is recognized and selected ads are displayed to you
       based on the information stored in this cookie.`,
      default: true,
    },
    analytics: {
      title: 'Analytics',
      description: `Modrinth uses in-house tools that allows us to get insights on how
       each user is using the platform, to improve the experience for
       everyone.\n By enabling this toggle, you allow us to get information across requests,
       disabling it will just remove all PII from now on.`,
      default: true,
    },
  },
}
