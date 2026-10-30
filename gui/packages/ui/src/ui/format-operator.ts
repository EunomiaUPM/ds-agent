export const formatOperator = (text: any) => {
  const clean = String(text)
    .replace(/[()[\]{},\"]/g, " ")
    .trim();
  const map: Record<string, string> = {
    gteq: "greater or equal than",
    lteq: "less or equal than",
    eq: "is equal to",
    neq: "is not equal to",
    gt: "greater than",
    lt: "less than",
    isA: "is a",
  };
  if (map[clean]) return map[clean];
  const spaced = clean.replace(/([A-Z])/g, " $1").trim();
  return spaced.toLowerCase();
};
