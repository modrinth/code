import { defineMessages } from '@vintl/vintl'

export default defineMessages({
  verifyExternalLinksTitle: {
    id: 'nags.verify-external-links.title',
    defaultMessage: 'Verify external links',
  },
  verifyExternalLinksDescription: {
    id: 'nags.verify-external-links.description',
    defaultMessage:
      "Some of your external links may be using domains that aren't recognized as common for their link type.",
  },
  invalidLicenseUrlTitle: {
    id: 'nags.invalid-license-url.title',
    defaultMessage: 'Invalid license URL',
  },
  invalidLicenseUrlDescriptionDefault: {
    id: 'nags.invalid-license-url.description.default',
    defaultMessage: 'License URL is invalid.',
  },
  invalidLicenseUrlDescriptionDomain: {
    id: 'nags.invalid-license-url.description.domain',
    defaultMessage:
      'Your license URL points to {domain}, which is not appropriate for license information. License URLs should link to the actual license text or legal documentation, not social media, gaming platforms etc.',
  },
  invalidLicenseUrlDescriptionMalformed: {
    id: 'nags.invalid-license-url.description.malformed',
    defaultMessage:
      'Your license URL appears to be malformed. Please provide a valid URL to your license text.',
  },
  gplLicenseSourceRequiredTitle: {
    id: 'nags.gpl-license-source-required.title',
    defaultMessage: 'GPL license requires source',
  },
  gplLicenseSourceRequiredDescription: {
    id: 'nags.gpl-license-source-required.description',
    defaultMessage:
      'Your {projectType} uses a GPL license which requires source code to be available. Please provide a source code link or consider using a different license.',
  },
  visitLinksSettingsTitle: {
    id: 'nags.visit-links-settings.title',
    defaultMessage: 'Visit links settings',
  },
  editLicenseTitle: {
    id: 'nags.edit-license.title',
    defaultMessage: 'Edit license',
  },
})
