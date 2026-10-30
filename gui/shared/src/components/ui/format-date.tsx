import dayjs from "dayjs";
import relativeTime from "dayjs/plugin/relativeTime";
import React from "react";

dayjs.extend(relativeTime);

/** The one date-time format of the GUI: sortable and free of day/month ambiguity. */
export const DATE_TIME_FORMAT = "YYYY-MM-DD HH:mm";
export const DATE_FORMAT = "YYYY-MM-DD";

interface FormatDateProps {
  date: string | Date | undefined | null;
  format?: string;
}

export const FormatDate = ({ date, format = DATE_TIME_FORMAT }: FormatDateProps) => {
  if (!date) return <span className="text-ink/35">—</span>;
  return <time dateTime={dayjs(date).toISOString()} title={dayjs(date).format("YYYY-MM-DD HH:mm:ss")}>{dayjs(date).format(format)}</time>;
};

/** "3 minutes ago", with the exact time on hover. */
export const RelativeTime = ({ date, className }: { date: string | Date; className?: string }) => (
  <time
    dateTime={dayjs(date).toISOString()}
    title={dayjs(date).format("YYYY-MM-DD HH:mm:ss")}
    className={className}
  >
    {dayjs(date).fromNow()}
  </time>
);
