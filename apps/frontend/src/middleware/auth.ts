const whitelistedParams = ["flow", "error"];

export default defineNuxtRouteMiddleware(async (_to, from) => {
  const config = useRuntimeConfig();
  const auth = await useAuth();

  if (auth.value.user) return;

  const fullPath = from.fullPath;

  const url = new URL(fullPath, config.public.apiBaseUrl);

  const extractedParams = Object.create(null) as Record<string, string>;

  for (const param of whitelistedParams) {
    const val = url.searchParams.get(param);
    if (val != null) {
      extractedParams[param] = val;
      url.searchParams.delete(param);
    }
  }

  return await navigateTo(
    {
      path: "/auth/sign-in",
      query: {
        redirect: `${url.pathname}${url.search}`,
        ...extractedParams,
      },
    },
    { replace: true },
  );
});
