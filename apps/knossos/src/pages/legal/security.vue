<template>
  <div class="markdown-body">
    <h1>Security Notice</h1>

    <p>
      This is the security notice for all Modrinth repositories. The notice explains how
      vulnerabilities should be reported.
    </p>
    <h2>Reporting a Vulnerability</h2>
    <p>
      If you've found a vulnerability, we would like to know so we can fix it before it is released
      publicly.
      <strong>Do not open a GitHub issue for a found vulnerability</strong>.
    </p>
    <p>
      Send details to <a href="mailto:jai@modrinth.com">jai@modrinth.com</a>
      including:
    </p>
    <ul>
      <li>the website, page or repository where the vulnerability can be observed</li>
      <li>a brief description of the vulnerability</li>
      <li>
        optionally the type of vulnerability and any related
        <a href="https://www.owasp.org/index.php/Category:OWASP_Top_Ten_2017_Project">
          OWASP category
        </a>
      </li>
      <li>non-destructive exploitation details</li>
    </ul>
    <p>We will do our best to reply as fast as possible.</p>
    <h2>Scope</h2>
    <p>The following vulnerabilities <strong>are not</strong> in scope:</p>
    <ul>
      <li>
        volumetric vulnerabilities, for example overwhelming a service with a high volume of
        requests
      </li>
      <li>
        reports indicating that our services do not fully align with "best practice", for example
        missing security headers
      </li>
    </ul>
    <p>If you aren't sure, you can still reach out via email or direct message.</p>
    <hr />
    <p>
      This notice is inspired by the
      <a href="https://www.pythondiscord.com/pages/security-notice/">
        Python Discord Security Notice</a
      >.
    </p>
    <p><em>Version 2022-11</em></p>
  </div>
</template>

<script setup>
const description =
  'The Security Notice of Modrinth, an open source modding platform focused on Minecraft.'

useSeoMeta({
  title: 'Security Notice - Modrinth',
  description,
  ogTitle: 'Security Notice',
  ogDescription: description,
})
</script>
