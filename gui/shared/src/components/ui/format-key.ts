export const formatKey = (text: any) => {
  const clean = String(text).replace(/[()[\]{},\"]/g, " ").trim();
  const spaced = clean.replace(/([A-Z])/g, ' $1').trim();
  return spaced.charAt(0).toUpperCase() + spaced.slice(1);
};
