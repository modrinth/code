export const useUserCountry = () =>
  useState("userCountry", () => {
    const headers = useRequestHeaders(["cf-ipcountry"]);

    return headers["cf-ipcountry"] ?? "US";
  });
