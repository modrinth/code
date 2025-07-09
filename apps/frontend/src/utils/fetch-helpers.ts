function segmentData<T>(data: T[], segmentSize: number): T[][] {
  return data.reduce((acc: T[][], curr, index) => {
    const segment = Math.floor(index / segmentSize);

    if (!acc[segment]) {
      acc[segment] = [];
    }
    acc[segment].push(curr);
    return acc;
  }, []);
}

export function fetchSegmented<T>(
  data: T[],
  createUrl: (ids: T[]) => string,
  options = {},
  segmentSize = 800,
): Promise<any> {
  return Promise.all(
    segmentData(data, segmentSize).map((ids) => useBaseFetch(createUrl(ids), options)),
  ).then((results) => results.flat());
}

export function asEncodedJsonArray<T>(data: T[]): string {
  return encodeURIComponent(JSON.stringify(data));
}
