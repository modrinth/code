const getLoaderVersions = async (loader: string) => {
  const loaderVersions = await fetch(
    `https://launcher-meta.modrinth.com/${loader?.toLowerCase()}/v0/manifest.json`,
  );
  return loaderVersions.json();
};

const getLoaderVersion = async (loader: string, version: string) => {
  const loaderVersion = await fetch(
    `https://launcher-meta.modrinth.com/${loader?.toLowerCase()}/v0/versions/${version}.json`,
  );
  return loaderVersion.json();
};

export default defineEventHandler(async (e) => {
  const params = new URLSearchParams(e._path?.split("?")[1] ?? "");
  if (!params.has("loader"))
    return new Response(
      JSON.stringify({
        error: "Missing loader",
      }),
      { status: 400, headers: { "Content-Type": "application/json" } },
    );
  const loader = params.get("loader");
  const version = params.get("version");
  if (version) {
    const loaderVersion = await getLoaderVersion(loader!, version);
    return new Response(JSON.stringify(loaderVersion), {
      headers: { "Content-Type": "application/json" },
    });
  }
  const loaderVersions = await getLoaderVersions(loader!);
  return new Response(JSON.stringify(loaderVersions), {
    headers: { "Content-Type": "application/json" },
  });
});
