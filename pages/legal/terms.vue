<template>
  <div>
    <h1>Terms and Conditions</h1>

    <h2>1. Terms</h2>

    <p>
      By accessing this Website, accessible from https://modrinth.com, you are
      agreeing to be bound by these Website Terms and Conditions of Use and
      agree that you are responsible for the agreement with any applicable local
      laws. If you disagree with any of these terms, you are prohibited from
      accessing this site. The materials contained in this Website are protected
      by copyright and trade mark law.
    </p>

    <h2>2. Use License</h2>

    <p>
      Permission is granted to temporarily download one copy of the materials on
      Rinth, Inc.'s Website for personal, non-commercial transitory viewing
      only. This is the grant of a license, not a transfer of title, and under
      this license you may not:
    </p>

    <ul>
      <li>modify or copy the materials;</li>
      <li>
        use the materials for any commercial purpose or for any public display;
      </li>
      <li>
        attempt to reverse engineer any software contained on Rinth, Inc.'s
        Website;
      </li>
      <li>
        remove any copyright or other proprietary notations from the materials;
        or
      </li>
      <li>
        transferring the materials to another person or "mirror" the materials
        on any other server.
      </li>
    </ul>

    <p>
      This will let Rinth, Inc. to terminate upon violations of any of these
      restrictions. Upon termination, your viewing right will also be terminated
      and you should destroy any downloaded materials in your possession whether
      it is printed or electronic format.
    </p>

    <h2>3. Disclaimer</h2>

    <p>
      All the materials on Rinth, Inc.’s Website are provided "as is". Rinth,
      Inc. makes no warranties, may it be expressed or implied, therefore
      negates all other warranties. Furthermore, Rinth, Inc. does not make any
      representations concerning the accuracy or reliability of the use of the
      materials on its Website or otherwise relating to such materials or any
      sites linked to this Website.
    </p>

    <h2>4. Limitations</h2>

    <p>
      Rinth, Inc. or its suppliers will not be hold accountable for any damages
      that will arise with the use or inability to use the materials on Rinth,
      Inc.’s Website, even if Rinth, Inc. or an authorize representative of this
      Website has been notified, orally or written, of the possibility of such
      damage. Some jurisdiction does not allow limitations on implied warranties
      or limitations of liability for incidental damages, these limitations may
      not apply to you.
    </p>

    <h2>5. Revisions and Errata</h2>

    <p>
      The materials appearing on Rinth, Inc.’s Website may include technical,
      typographical, or photographic errors. Rinth, Inc. will not promise that
      any of the materials in this Website are accurate, complete, or current.
      Rinth, Inc. may change the materials contained on its Website at any time
      without notice. Rinth, Inc. does not make any commitment to update the
      materials.
    </p>

    <h2>6. Links</h2>

    <p>
      Rinth, Inc. has not reviewed all of the sites linked to its Website and is
      not responsible for the contents of any such linked site. The presence of
      any link does not imply endorsement by Rinth, Inc. of the site. The use of
      any linked website is at the user’s own risk.
    </p>

    <h2>7. Site Terms of Use Modifications</h2>

    <p>
      Rinth, Inc. may revise these Terms of Use for its Website at any time
      without prior notice. By using this Website, you are agreeing to be bound
      by the current version of these Terms and Conditions of Use.
    </p>

    <h2>8. Your Privacy</h2>

    <p>
      Please read our
      <nuxt-link to="/legal/privacy"> Privacy Policy</nuxt-link>.
    </p>

    <h2>9. Governing Law</h2>

    <p>
      Any claim related to Rinth, Inc.'s Website shall be governed by the laws
      of us without regards to its conflict of law provisions.
    </p>

    <h2>10. Content</h2>

    <p>
      When you upload text, software, mods, scripts, graphics, photos, audio,
      videos, links, interactive features and other materials that may be viewed
      on or accessed through Modrinth, we refer to it as "Content".
    </p>

    <ul>
      <li>
        You are responsible for all activity and Content that is uploaded under
        your Modrinth account.
      </li>
      <li>
        You retain all of your ownership rights to your Content. We do not claim
        any ownership in or to any of your Content.
      </li>
      <li>
        To enable us to provide the services of Modrinth, you hereby grant us a
        worldwide, non-exclusive, royalty-free, and unrestricted license to use,
        reproduce, distribute copies, prepare derivative works of, or display
        Content in connection with Modrinth in any medium and for any purpose
        (including commercial purposes).
      </li>
    </ul>

    <p>
      All Content on Modrinth must obey the
      <nuxt-link to="/legal/rules">Content Rules</nuxt-link>. Please be aware of
      these Rules before uploading any Content to Modrinth.
    </p>
  </div>
</template>

<script>
export default {
  auth: false,
  head: {
    title: 'Terms - Modrinth',
    meta: [
      {
        hid: 'description',
        name: 'description',
        content:
          'The Terms and Conditions of Modrinth, an open source modding platform focused on Minecraft.',
      },
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Terms and Conditions',
      },
      {
        hid: 'og:title',
        name: 'og:title',
        content: 'Terms and Conditions',
      },
      {
        hid: 'og:url',
        name: 'og:url',
        content: `https://modrinth.com/legal/terms`,
      },
    ],
  },
}
</script>

<style lang="scss" scoped></style>
