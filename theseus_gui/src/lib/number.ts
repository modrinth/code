/**
 * Convert large numbers to human readable strings
 * @source https://github.com/rohmanhm/simplify-number
 */
export function simplify(num = 0): string {
  let numberVar = num;

  // 2 decimal places => 100, 3 => 1000, etc
  const decPlaces = Math.pow(10, 1);

  // Enumerate number abbreviations
  const abbrev = ['K', 'M', 'B', 'T'];

  // Go through the array backwards, so we do the largest first
  for (let i = abbrev.length - 1; i >= 0; i--) {
    // Convert array index to "1000", "1000000", etc
    const size = Math.pow(10, (i + 1) * 3);

    // If the number is bigger or equal do the abbreviation
    if (size <= numberVar) {
      // Here, we multiply by decPlaces, round, and then divide by decPlaces.
      // This gives us nice rounding to a particular decimal place.
      numberVar = Math.round((numberVar * decPlaces) / size) / decPlaces;

      // Handle special case where we round up to the next abbreviation
      if (numberVar === 1000 && i < abbrev.length - 1) {
        numberVar = 1;
        i++;
      }

      // Add the letter for the abbreviation
      (numberVar as any) += abbrev[i];

      // We are done... stop
      break;
    }
  }

  return String(numberVar);
}
