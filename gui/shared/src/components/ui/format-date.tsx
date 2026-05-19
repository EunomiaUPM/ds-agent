import dayjs from "dayjs";
import React from "react";

interface FormatDateProps {
  date: string | Date | undefined | null;
  format?: string;
}

export const FormatDate = ({ date, format = "DD/MM/YYYY - HH:mm" }: FormatDateProps) => {
  if (!date) return <span className="text-gray-400">-</span>;
  return <span>{dayjs(date).format(format)}</span>;
};
