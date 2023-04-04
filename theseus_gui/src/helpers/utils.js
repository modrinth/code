export const releaseColor = (releaseType) => {
  switch (releaseType) {
    case "release":
      return "green";
    case "beta":
      return "orange";
    case "alpha":
      return "red";
    default:
      return "";
  }
}
