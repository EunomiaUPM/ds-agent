export const formatValue = (text: any) => {
  const clean = String(text)
    .replace(/[()[\]{},\"]/g, " ")
    .trim();
  const datePattern = /^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}:\d{2}(.\d+)?(Z|[+-]\d{2}:\d{2})?)?$/;
  if (datePattern.test(clean)) {
    const d = new Date(clean);
    if (!isNaN(d.getTime())) {
      // e.g. 2025-01-01
      return d.toLocaleDateString("en-CA"); // YYYY-MM-DD
    }
  }
  return clean;
};
