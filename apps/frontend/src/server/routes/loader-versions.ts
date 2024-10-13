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
  const loaderVersions = await fetch(
    `https://launcher-meta.modrinth.com/${loader?.toLowerCase()}/v0/manifest.json`,
  );

  return new Response(JSON.stringify(await loaderVersions.json()), {
    headers: { "Content-Type": "application/json" },
  });
});
