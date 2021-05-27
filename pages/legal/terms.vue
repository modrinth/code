<template>
  <div class="main">
    <div class="container">
      <h1>Terms and Conditions</h1>

      <h2>1. Terms</h2>

      <p>
        By accessing this Website, accessible from https://modrinth.com, you are
        agreeing to be bound by these Website Terms and Conditions of Use and
        agree that you are responsible for the agreement with any applicable
        local laws. If you disagree with any of these terms, you are prohibited
        from accessing this site. The materials contained in this Website are
        protected by copyright and trade mark law.
      </p>

      <h2>2. Use License</h2>

      <p>
        Permission is granted to temporarily download one copy of the materials
        on Guavy LLC's Website for personal, non-commercial transitory viewing
        only. This is the grant of a license, not a transfer of title, and under
        this license you may not:
      </p>

      <ul>
        <li>modify or copy the materials;</li>
        <li>
          use the materials for any commercial purpose or for any public
          display;
        </li>
        <li>
          attempt to reverse engineer any software contained on Guavy LLC's
          Website;
        </li>
        <li>
          remove any copyright or other proprietary notations from the
          materials; or
        </li>
        <li>
          transferring the materials to another person or "mirror" the materials
          on any other server.
        </li>
      </ul>

      <p>
        This will let Guavy LLC to terminate upon violations of any of these
        restrictions. Upon termination, your viewing right will also be
        terminated and you should destroy any downloaded materials in your
        possession whether it is printed or electronic format.
      </p>

      <h2>3. Disclaimer</h2>

      <p>
        All the materials on Guavy LLC’s Website are provided "as is". Guavy LLC
        makes no warranties, may it be expressed or implied, therefore negates
        all other warranties. Furthermore, Guavy LLC does not make any
        representations concerning the accuracy or reliability of the use of the
        materials on its Website or otherwise relating to such materials or any
        sites linked to this Website.
      </p>

      <h2>4. Limitations</h2>

      <p>
        Guavy LLC or its suppliers will not be hold accountable for any damages
        that will arise with the use or inability to use the materials on Guavy
        LLC’s Website, even if Guavy LLC or an authorize representative of this
        Website has been notified, orally or written, of the possibility of such
        damage. Some jurisdiction does not allow limitations on implied
        warranties or limitations of liability for incidental damages, these
        limitations may not apply to you.
      </p>

      <h2>5. Revisions and Errata</h2>

      <p>
        The materials appearing on Guavy LLC’s Website may include technical,
        typographical, or photographic errors. Guavy LLC will not promise that
        any of the materials in this Website are accurate, complete, or current.
        Guavy LLC may change the materials contained on its Website at any time
        without notice. Guavy LLC does not make any commitment to update the
        materials.
      </p>

      <h2>6. Links</h2>

      <p>
        Guavy LLC has not reviewed all of the sites linked to its Website and is
        not responsible for the contents of any such linked site. The presence
        of any link does not imply endorsement by Guavy LLC of the site. The use
        of any linked website is at the user’s own risk.
      </p>

      <h2>7. Site Terms of Use Modifications</h2>

      <p>
        Guavy LLC may revise these Terms of Use for its Website at any time
        without prior notice. By using this Website, you are agreeing to be
        bound by the current version of these Terms and Conditions of Use.
      </p>

      <h2>8. Your Privacy</h2>

      <p>
        Please read our
        <nuxt-link to="/legal/privacy"> Privacy Policy</nuxt-link>.
      </p>

      <h2>9. Governing Law</h2>

      <p>
        Any claim related to Guavy LLC's Website shall be governed by the laws
        of us without regards to its conflict of law provisions.
      </p>

      <h2>10. Content</h2>

      <p>
        When you upload text, software, mods, scripts, graphics, photos, audio,
        videos, links, interactive features and other materials that may be
        viewed on, or accessed through Modrinth, we refer to it as “Content”.
      </p>

      <ul>
        <li>
          You must own or have the necessary licenses, rights, consents, and
          permissions to store, share or distribute the Content that is uploaded
          under your Modrinth account.
        </li>
        <li>
          You are responsible for all activity and Content that is uploaded
          under your Modrinth account.
        </li>
        <li>
          You must not transmit any viruses, worms, malware, or any other code
          of a destructive nature through Modrinth.
        </li>
        <li>
          You retain all of your ownership rights to your Content. We do not
          claim any ownership in or to any of your Content.
        </li>
        <li>
          To enable us to provide the services of Modrinth, you hereby grant us
          a worldwide, non-exclusive, royalty-free, and unrestricted license to
          use, reproduce, distribute copies, prepare derivative works of, or
          display Content in connection with Modrinth in any medium and for any
          purpose (including commercial purposes).
        </li>
      </ul>
    </div>
    <m-footer class="footer" centered />
  </div>
</template>

<script>
import MFooter from '~/components/layout/MFooter'

export default {
  components: {
    MFooter,
  },
  auth: false,
  head: {
    title: 'Terms - Modrinth',
    meta: [
      {
        hid: 'description',
        name: 'description',
        content:
          'The Terms of Service of Modrinth, an open source modding platform. Modrinth currently supports Minecraft, including the forge and fabric mod loaders.',
      },
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Terms of Service',
      },
      {
        hid: 'og:title',
        name: 'og:title',
        content: 'Terms of Service',
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

<style lang="scss" scoped>
.main {
  margin: var(--spacing-card-sm) auto;
  max-width: 800px;
}

.container {
  @extend %card;
  padding: var(--spacing-card-sm) var(--spacing-card-lg);
}

a {
  text-decoration: underline;
  color: var(--color-link);
}
</style>
