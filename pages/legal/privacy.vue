<template>
  <div class="main">
    <div class="card">
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

      <h2>General Data Protection Regulation (GDPR)</h2>
      <p>We (Modrinth) are a Data Controller of your information.</p>

      <p>
        Rinth, Inc. legal basis for collecting and using the personal
        information described in this Privacy Policy depends on the Personal
        Information we collect and the specific context in which we collect the
        information:
      </p>
      <ul>
        <li>Rinth, Inc. needs to perform a contract with you</li>
        <li>You have given Rinth, Inc. permission to do so</li>
        <li>
          Processing your personal information is in Rinth, Inc. legitimate
          interests
        </li>
        <li>Rinth, Inc. needs to comply with the law</li>
      </ul>

      <p>
        Rinth, Inc. will retain your personal information only for as long as is
        necessary for the purposes set out in this Privacy Policy. We will
        retain and use your information to the extent necessary to comply with
        our legal obligations, resolve disputes, and enforce our policies.
      </p>

      <p>
        If you are a resident of the European Economic Area (EEA), you have
        certain data protection rights. If you wish to be informed what Personal
        Information we hold about you and if you want it to be removed from our
        systems, please contact us.
      </p>

      <p>
        In certain circumstances, you have the following data protection rights:
      </p>
      <ul>
        <li>
          The right to access, update or to delete the information we have on
          you. (please email admin@modrinth.com for data requests)
        </li>
        <li>The right of rectification.</li>
        <li>The right to object.</li>
        <li>The right of restriction.</li>
        <li>The right to data portability</li>
        <li>The right to withdraw consent</li>
      </ul>

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
        websites.
      </p>

      <p>
        In addition, anytime you click on links (including advertising banners),
        which take you to third-party websites, you will be subject to the third
        parties’ privacy policies. While we support the protection of our
        customer’s privacy on the Internet, Modrinth expressly disclaims any and
        all liability for the actions of third parties, including but without
        limitation to actions relating to the use and/or disclosure of personal
        information by third parties.
      </p>

      <p>
        Modrinth makes use of external ads providers. These include EthicalAds.
        You can read EthicalAds's privacy policy here and the data they collect:
        <a href="https://www.ethicalads.io/privacy-policy/"
          >https://www.ethicalads.io/privacy-policy/</a
        >
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
  </div>
</template>

<script>
export default {
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

a {
  text-decoration: underline;
  color: var(--color-link);
}
</style>
