/** @type {import('./$types').LayoutServerLoad} */
export async function load({ request }) {
  return {
    acceptLanguage: await request.headers.get('Accept-Language')
  };
}