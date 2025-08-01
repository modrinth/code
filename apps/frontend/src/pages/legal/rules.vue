<template>
  <div class="markdown-body">
    <h1>Content Rules</h1>

    <p>
      These Content Rules are to be considered part of our
      <nuxt-link to="/legal/terms">Terms of Use</nuxt-link> and apply to any and all User
      Contributions, Gaming Content, and use of Interactive Services (collectively, "Content").
    </p>

    <p>
      If you find any violations of these Rules on our website, you should make us aware. You may
      use the Report button on any project, version, or user page, or you may email us at
      <a href="mailto:support@modrinth.com">support@modrinth.com</a>.
    </p>

    <h2 id="prohibited-content">1. Prohibited Content</h2>

    <p>
      Content must in their entirety comply with all applicable federal, state, local, and
      international laws and regulations. Without limiting the foregoing, Content must not:
    </p>
    <ol>
      <li>
        Contain any material which is defamatory, obscene, indecent, abusive, offensive, harassing,
        violent, hateful, inflammatory, harmful, damaging, disruptive, contradictory, or otherwise
        objectionable.
      </li>
      <li>
        Promote sexually explicit or pornographic material, violence, or discrimination based on
        race, sex, gender, religion, nationality, disability, sexual orientation, or age.
      </li>
      <li>
        Infringe any patent, trademark, trade secret, copyright, or other intellectual property or
        other rights of any other person.
      </li>
      <li>
        Violate the legal rights (including the rights of publicity and privacy) of others or
        contain any material that could give rise to any civil or criminal liability under
        applicable laws or regulations or that otherwise may be in conflict with our
        <nuxt-link to="/legal/terms">Terms of Use</nuxt-link> or
        <nuxt-link to="/legal/privacy">Privacy Policy</nuxt-link>.
      </li>
      <li>
        Promote any illegal activity, or advocate, promote or assist any unlawful act, including
        real-life drugs or illicit substances.
      </li>
      <li>
        Cause annoyance, inconvenience, or needless anxiety or be likely to upset, embarrass, alarm,
        annoy, harm, or deceive any other person.
      </li>
      <li>Make or share intentionally wrong or misleading claims.</li>
      <li>
        Impersonate any person, or misrepresent your identity or affiliation with any person or
        organization.
      </li>
      <li>
        Give the impression that they emanate from or are endorsed by us or any other person or
        entity, if this is not the case.
      </li>
      <li>Contain an excessive amount of profane language.</li>
      <li>
        Be designed to upload any data to a remote server (i.e. one that the user does not directly
        choose to connect to in-game) without clear disclosure.
      </li>
      <li>
        Bypass restrictions placed by Mojang to prevent users from joining certain in-game servers.
      </li>
    </ol>

    <h2 id="clear-and-honest-function">2. Clear and Honest Function</h2>

    <p>
      Projects, a form of Content, must make a clear and honest attempt to describe their purpose in
      designated areas on the project page. Necessary information must not be obscured in any way.
      Using confusing language or technical jargon when it is not necessary constitutes a violation.
    </p>

    <h3 id="general-expectations">2.1. General Expectations</h3>

    <p>
      From a project description, users should be able to understand what the project does and how
      to use it. Projects must attempt to describe the following three things within their
      description:
    </p>
    <ol type="a">
      <li>what the project specifically does or adds</li>
      <li>why someone should want to download the project</li>
      <li>any other critical information the user must know before downloading</li>
    </ol>

    <h3 id="accessibility">2.2. Accessibility</h3>

    <p>
      Project descriptions must be accessible so that they can be read through a variety of mediums.
      All descriptions must have a plain-text version, though images, videos, and other content can
      take priority if desired. Headers must not be used for body text.
    </p>

    <p>
      Project descriptions must have an English-language translation unless they are exclusively
      meant for use in a specific language, such as translation packs. Descriptions may provide
      translations into other languages if desired.
    </p>

    <h2 id="cheats-and-hacks">3. Cheats and Hacks</h2>

    <p>
      Projects cannot contain or download "cheats", which we define as a client-side modification
      that:
    </p>
    <ol>
      <li>is advertised as a "cheat", "hack", or "hacked client"</li>
      <li>
        gives an unfair advantage in a multiplayer setting over other players that do not have a
        comparable modification and does not provide a server-side opt-out
      </li>
      <li>
        contains any of the following functions without requiring a server-side opt-in:
        <ol type="a">
          <li>X-ray or the ability to see through opaque blocks</li>
          <li>aim bot or aim assist</li>
          <li>flight, speed, or other movement modifications</li>
          <li>automatic or assisted PvP combat</li>
          <li>
            active client-side hiding of third party modifications that have server-side opt-outs
          </li>
          <li>item duplication</li>
        </ol>
      </li>
    </ol>

    <h2 id="copyright-and-legality-of-content">4. Copyright and Reuploads</h2>

    <p>
      You must own or have the necessary licenses, rights, consents, and permissions to store,
      share, and distribute the Content that is uploaded under your Modrinth account.
    </p>

    <p>
      Content may not be directly reuploaded from another source without explicit permission from
      the original author. If explicit permission has been granted, or it is a license-abiding
      "fork", this restriction does not apply. We define "forks" as modified copies of a project
      which have diverged substantially from the original project.
    </p>

    <h2 id="miscellaneous">5. Miscellaneous</h2>

    <p>
      There are certain other small aspects to creating projects that all authors should attempt to
      abide by. These will not necessarily always be enforced, but abiding by all will result in a
      faster review with fewer potential issues.
    </p>
    <ol>
      <li>
        All metadata, including license, client/server-side information, tags, etc. are filled out
        correctly and are consistent with information found elsewhere.
      </li>
      <li>
        Project titles are only the name of the project, without any other unnecessary filler data.
      </li>
      <li>
        Project summaries contain a small summary of the project without any formatting and without
        repeating the project title.
      </li>
      <li>All external links lead to public resources that are relevant.</li>
      <li>Gallery images are relevant to the project and each contain a title.</li>
      <li>All dependencies must be specified in the Dependencies section of each version.</li>
      <li>
        "Additional files" are only used for special designated purposes, such as source JAR files.
        In other words, separate versions and/or projects are used where appropriate instead of
        additional files.
      </li>
    </ol>
  </div>
</template>

<script setup>
const description =
  'The Content Rules of Modrinth, an open source modding platform focused on Minecraft.'

useSeoMeta({
  title: 'Content Rules - Modrinth',
  description,
  ogTitle: 'Content Rules',
  ogDescription: description,
})
</script>
