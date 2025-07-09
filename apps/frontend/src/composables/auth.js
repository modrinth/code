export const useAuth = async (oldToken = null) => {
  const auth = useState("auth", () => ({
    user: null,
    token: "",
    headers: {},
  }));

  if (!auth.value.user || oldToken) {
    auth.value = await initAuth(oldToken);
  }

  return auth;
};

export const initAuth = async (oldToken = null) => {
  const auth = {
    user: null,
    token: "",
  };

  if (oldToken === "none") {
    return auth;
  }

  const route = useRoute();
  const authCookie = useCookie("auth-token", {
    maxAge: 60 * 60 * 24 * 365 * 10,
    sameSite: "lax",
    secure: true,
    httpOnly: false,
    path: "/",
  });

  if (oldToken) {
    authCookie.value = oldToken;
  }

  if (route.query.code && !route.fullPath.includes("new_account=true")) {
    authCookie.value = route.query.code;
  }

  if (route.fullPath.includes("new_account=true") && route.path !== "/auth/welcome") {
    const redirect = route.path.startsWith("/auth/") ? null : route.fullPath;

    await navigateTo(
      `/auth/welcome?authToken=${route.query.code}${
        redirect ? `&redirect=${encodeURIComponent(redirect)}` : ""
      }`,
    );
  }

  if (authCookie.value) {
    auth.token = authCookie.value;

    if (!auth.token || !auth.token.startsWith("mra_")) {
      return auth;
    }

    try {
      auth.user = await useBaseFetch(
        "user",
        {
          headers: {
            Authorization: auth.token,
          },
        },
        true,
      );
    } catch {
      /* empty */
    }
  }

  if (!auth.user && auth.token) {
    try {
      const session = await useBaseFetch(
        "session/refresh",
        {
          method: "POST",
          headers: {
            Authorization: auth.token,
          },
        },
        true,
      );

      auth.token = session.session;
      authCookie.value = auth.token;

      auth.user = await useBaseFetch(
        "user",
        {
          headers: {
            Authorization: auth.token,
          },
        },
        true,
      );
    } catch {
      authCookie.value = null;
    }
  }

  return auth;
};

export const getAuthUrl = (provider, redirect = "/dashboard") => {
  const config = useRuntimeConfig();
  const route = useNativeRoute();

  const fullURL = route.query.launcher
    ? "https://launcher-files.modrinth.com"
    : `${config.public.siteUrl}/auth/sign-in?redirect=${redirect}`;

  return `${config.public.apiBaseUrl}auth/init?provider=${provider}&url=${encodeURIComponent(fullURL)}`;
};

export const removeAuthProvider = async (provider) => {
  startLoading();
  try {
    const auth = await useAuth();

    await useBaseFetch("auth/provider", {
      method: "DELETE",
      body: {
        provider,
      },
    });
    await useAuth(auth.value.token);
  } catch (err) {
    const data = useNuxtApp();
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data.description,
      type: "error",
    });
  }
  stopLoading();
};
