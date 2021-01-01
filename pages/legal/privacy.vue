<template>
  <div class="main">
    <div class="container">
      <h1>Privacy Policy</h1>

      <p>
        At Modrinth, accessible from https://modrinth.com, one of our main
        priorities is the privacy of our visitors. This Privacy Policy document
        contains types of information that is collected and recorded by Modrinth
        and how we use it.
      </p>

      <p>
        If you have additional questions or require more information about our
        Privacy Policy, do not hesitate to contact us.
      </p>

      <h2>Log Files</h2>

      <p>
        Modrinth follows a standard procedure of using log files. These files
        log visitors when they visit websites. All hosting companies do this and
        a part of hosting services' analytics. The information collected by log
        files include internet protocol (IP) addresses, browser type, Internet
        Service Provider (ISP), date and time stamp, referring/exit pages, and
        possibly the number of clicks. These are not linked to any information
        that is personally identifiable. The purpose of the information is for
        analyzing trends, administering the site, tracking users' movement on
        the website, and gathering demographic information.
      </p>

      <h2>Cookies and Web Beacons</h2>

      <p>
        Like any other website, Modrinth uses 'cookies'. These cookies are used
        to store information including visitors' preferences, and the pages on
        the website that the visitor accessed or visited. The information is
        used to optimize the users' experience by customizing our web page
        content based on visitors' browser type and/or other information.
      </p>

      <p>
        For more general information on cookies, please read
        <a href="https://www.privacypolicies.com/blog/cookies/"
          >"What Are Cookies"</a
        >.
      </p>

      <h2>Our Advertising Partners</h2>

      <p>
        Some of advertisers on our site may use cookies and web beacons. Our
        advertising partners are listed below. Each of our advertising partners
        has their own Privacy Policy for their policies on user data. For easier
        access, we hyperlinked to their Privacy Policies below.
      </p>

      <ul>
        <li>
          <p>EthicalAds</p>
          <p>
            <a href="https://www.ethicalads.io/privacy-policy/"
              >https://www.ethicalads.io/privacy-policy/</a
            >
          </p>
        </li>
      </ul>

      <h2>Privacy Policies</h2>

      <P
        >You may consult this list to find the Privacy Policy for each of the
        advertising partners of Modrinth.</P
      >

      <p>
        Third-party ad servers or ad networks uses technologies like cookies,
        JavaScript, or Web Beacons that are used in their respective
        advertisements and links that appear on Modrinth, which are sent
        directly to users' browser. They automatically receive your IP address
        when this occurs. These technologies are used to measure the
        effectiveness of their advertising campaigns and/or to personalize the
        advertising content that you see on websites that you visit.
      </p>

      <p>
        Note that Modrinth has no access to or control over these cookies that
        are used by third-party advertisers.
      </p>

      <h2>Third Party Privacy Policies</h2>

      <p>
        Modrinth's Privacy Policy does not apply to other advertisers or
        websites. Thus, we are advising you to consult the respective Privacy
        Policies of these third-party ad servers for more detailed information.
        It may include their practices and instructions about how to opt-out of
        certain options.
      </p>

      <p>
        You can choose to disable cookies through your individual browser
        options. To know more detailed information about cookie management with
        specific web browsers, it can be found at the browsers' respective
        websites. What Are Cookies?
      </p>

      <h2>Children's Information</h2>

      <p>
        Another part of our priority is adding protection for children while
        using the internet. We encourage parents and guardians to observe,
        participate in, and/or monitor and guide their online activity.
      </p>

      <p>
        Modrinth does not knowingly collect any Personal Identifiable
        Information from children under the age of 13. If you think that your
        child provided this kind of information on our website, we strongly
        encourage you to contact us immediately and we will do our best efforts
        to promptly remove such information from our records.
      </p>

      <h2>Online Privacy Policy Only</h2>

      <p>
        This Privacy Policy applies only to our online activities and is valid
        for visitors to our website with regards to the information that they
        shared and/or collect in Modrinth. This policy is not applicable to any
        information collected offline or via channels other than this website.
      </p>

      <h2>Consent</h2>

      <p>
        By using our website, you hereby consent to our Privacy Policy and agree
        to its Terms and Conditions.
      </p>
    </div>
    <m-footer class="footer" centered />
  </div>
</template>

<script>
import MFooter from '@/components/MFooter'

export default {
  components: {
    MFooter,
  },
  auth: false,
  head: {
    title: 'Privacy - Modrinth',
    meta: [
      {
        hid: 'description',
        name: 'description',
        content:
          'The privacy policy of Modrinth, an open source modding platform. Modrinth currently supports Minecraft, including the forge and fabric mod loaders.',
      },
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Privacy Policy',
      },
      {
        hid: 'og:title',
        name: 'og:title',
        content: 'Privacy Policy',
      },
      {
        hid: 'og:url',
        name: 'og:url',
        content: `https://modrinth.com/legal/privacy`,
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
</style>
