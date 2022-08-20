import { init, waitLocale, t, getLocaleFromAcceptLanguageHeader } from 'svelte-intl-precompile';
import { registerAll, availableLocales } from '$locales';

registerAll();

/** @type {import('./$types').PageLoad} */
export async function load({ parent }) {
  const { acceptLanguage } = await parent();  

  init({
    fallbackLocale: 'en',
    initialLocale: getLocaleFromAcceptLanguageHeader(acceptLanguage, availableLocales)
  });
  await waitLocale();

  return {};
};
