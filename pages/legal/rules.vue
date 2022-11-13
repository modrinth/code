<template>
  <div>
    <h1>Content Rules</h1>

    <p>
      In order to facilitate Modrinth's
      <nuxt-link to="/legal/terms">Terms and Conditions</nuxt-link>, all Content
      must obey the following Rules. For more information on what exactly
      Content is, please refer to the Content section of the Terms.
    </p>

    <p>
      Please note that these are general rules and will not be enforced "to the
      letter". We reserve the right to modify and/or remove any file, project,
      or other Content uploaded to our platform for any reason. We reserve the
      right to introduce new rules at any time, which may or may not
      retroactively apply to already uploaded Content at the discretion of our
      moderators.
    </p>

    <p>
      If you find any violations of these Rules on our website, it is your
      responsibility to report it. You may use the Report button on any project,
      version, or user page, or you may email us at
      <a href="mailto:support@modrinth.com">support@modrinth.com</a>.
    </p>

    <h2 id="malicious-content">1. Malicious Content</h2>

    <p>
      Content cannot contain or download malware, which we define as anything
      that is designed:
    </p>
    <ul>
      <li>
        to upload any data to a remote server (i.e. one that the user does not
        directly choose to connect to in-game) without clear disclosure
      </li>
      <li>
        to disrupt, damage, or otherwise cause harm or damage to an individual,
        computer, or network
      </li>
    </ul>

    <h2 id="clear-and-honest-function">2. Clear and honest function</h2>

    <p>
      Content, especially projects, must make a clear and honest attempt to
      describe their purpose on the page(s) where it may be found.
    </p>

    <p>
      Content must not make or share intentionally wrong or misleading claims.
      This includes but is not limited to claims regarding the Content itself,
      claims regarding other Content, and claims not relating to Content on
      Modrinth.
    </p>

    <h3 id="general-expectations">2.1. General expectations</h3>

    <p>
      Projects in particular must attempt to describe the following three things
      within their description:
    </p>
    <ul>
      <li>what a project specifically does or adds</li>
      <li>why someone should want to download the project</li>
      <li>
        any other critical information the user must know before downloading
      </li>
    </ul>

    <p>
      Project descriptions must also be accessible. For the most part, this
      means that descriptions cannot mostly consist of text within images, and
      necessary information cannot be obscured.
    </p>

    <p>
      Projects which don't meet of these expectations may be removed from search
      rather than removed from the platform altogether, at the moderators'
      discretion.
    </p>

    <h2 id="cheats-and-hacks">3. Cheats and Hacks</h2>

    <p>
      Projects cannot contain or download "cheats", which we define as a
      client-side modification that:
    </p>
    <ul>
      <li>is advertised as a "cheat", "hack", or "hacked client"</li>
      <li>
        gives an unfair advantage in a multiplayer setting over other players
        that do not have a comparable modification and does not provide a
        server-side opt-out
      </li>
      <li>
        contains any of the following functions without requiring a server-side
        opt-in:
        <ul>
          <li>X-ray or the ability to see through opaque blocks</li>
          <li>aim bot or aim assist</li>
          <li>flight, speed, or other movement modifications</li>
          <li>automatic PvP</li>
          <li>
            active client-side hiding of third party modifications that have
            server-side opt-outs
          </li>
          <li>item duplication</li>
        </ul>
      </li>
    </ul>

    <h2 id="copyright-and-legality-of-content">
      4. Copyright and legality of Content
    </h2>

    <p>
      You must own or have the necessary licenses, rights, consents, and
      permissions to store, share, or distribute the Content that is uploaded
      under your Modrinth account.
    </p>

    <p>
      Content may not be directly "reuploaded" from another platform without the
      permission of the author or copyright holder, even with the appropriate
      licensing or other rights. This restriction does not apply to content
      within modpacks or to so called "forks" - that is, modified copies of a
      project which have diverged substantially enough from the original
      project, at the discretion of Modrinth's moderators.
    </p>

    <p>
      Content must not infringe upon anyone's rights or intellectual property.
    </p>

    <p>
      Content must abide by the laws which govern Rinth, Inc., i.e. those of the
      United States and of the State of Delaware.
    </p>

    <h2 id="prohibited-content">5. Prohibited Content</h2>

    <p>
      Content on Modrinth is meant to be appropriate for audiences 13 years of
      age and above.
    </p>

    <p>This means that the following Content is not allowed:</p>
    <ul>
      <li>Content containing sexual or explicit material</li>
      <li>Content promoting or sharing harmful or hateful behavior</li>
      <li>
        Content with a focus on items such as alcohol, tobacco, and other drugs
      </li>
      <li>Content with an excessive amount of profane language</li>
    </ul>
  </div>
</template>

<script>
export default {
  auth: false,
  head: {
    title: 'Rules - Modrinth',
    meta: [
      {
        hid: 'description',
        name: 'description',
        content:
          'The Content Rules of Modrinth, an open source modding platform focused on Minecraft.',
      },
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Content Rules',
      },
      {
        hid: 'og:title',
        name: 'og:title',
        content: 'Content Rules',
      },
      {
        hid: 'og:url',
        name: 'og:url',
        content: `https://modrinth.com/legal/rules`,
      },
    ],
  },
}
</script>

<style lang="scss" scoped></style>
