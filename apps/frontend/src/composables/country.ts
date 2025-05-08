import { useState, useRequestHeaders } from "#imports";

export const useUserCountry = () => {
  const country = useState<string>("userCountry", () => "US");
  const fromServer = useState<boolean>("userCountryFromServer", () => false);

  if (import.meta.server) {
    const headers = useRequestHeaders(["cf-ipcountry", "accept-language"]);
    const cf = headers["cf-ipcountry"];
    if (cf) {
      country.value = cf.toUpperCase();
      fromServer.value = true;
    } else {
      const al = headers["accept-language"] || "";
      const tag = al.split(",")[0];
      const val = tag.split("-")[1]?.toLowerCase();
      if (val) {
        country.value = val;
        fromServer.value = true;
      }
    }
  }

  if (import.meta.client) {
    onMounted(() => {
      if (fromServer.value) return;
      const lang = navigator.language || navigator.userLanguage || "";
      const region = lang.split("-")[1];
      if (region) {
        country.value = region.toUpperCase();
      }
    });
  }

  return country;
};
