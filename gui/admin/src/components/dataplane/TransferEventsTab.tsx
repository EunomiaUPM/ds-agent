import React, { useMemo } from "react";
import { TabsContent } from "shared/src/components/ui/tabs";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Badge } from "shared/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "shared/components/ui/table";
import {
  ChartConfig,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "shared/components/ui/chart";
import { Bar, BarChart, CartesianGrid, XAxis, YAxis } from "recharts";
import { TransferEventDto } from "shared/src/data/orval/model";
import { EventData } from "./EventData";

const chartConfig = {
  count: { label: "Events", color: "#9DD5F2" },
} satisfies ChartConfig;

function useEventsChartData(events: TransferEventDto[]) {
  return useMemo(() => {
    if (!events.length) return [];

    const byMinuteMs = new Map<number, number>();
    events.forEach((event) => {
      if (!event.created_at) return;
      const ms =
        Math.floor(new Date(event.created_at).getTime() / 60_000) * 60_000;
      byMinuteMs.set(ms, (byMinuteMs.get(ms) ?? 0) + 1);
    });

    if (!byMinuteMs.size) return [];

    const timestamps = Array.from(byMinuteMs.keys());
    const minMs = Math.min(...timestamps);
    const maxMs = Math.max(...timestamps);
    const endMs = maxMs + 15 * 60_000;

    const result: { time: string; count: number }[] = [];
    for (let t = minMs; t <= endMs; t += 60_000) {
      const d = new Date(t);
      result.push({
        time: `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`,
        count: byMinuteMs.get(t) ?? 0,
      });
    }
    return result;
  }, [events]);
}

export function TransferEventsTab({ events }: { events: TransferEventDto[] }) {
  const chartData = useEventsChartData(events);

  return (
    <TabsContent value="transfer-events" className="w-full">
      {/* Activity chart */}
      <div className="mt-4 mb-4 rounded-md border border-white/10 bg-muted/20 p-4">
        <div className="text-xs font-semibold text-muted-foreground uppercase tracking-wide mb-3">
          Activity — Events per Minute
        </div>
        {chartData.length > 0 ? (
          <ChartContainer config={chartConfig} className="h-[200px] w-full">
            <BarChart
              data={chartData}
              margin={{ top: 8, right: 16, left: 0, bottom: 0 }}
            >
              <CartesianGrid vertical={false} strokeDasharray="3 3" />
              <XAxis
                dataKey="time"
                tickLine={false}
                axisLine={false}
                tickMargin={8}
                tick={{ fontSize: 11 }}
              />
              <YAxis
                tickLine={false}
                axisLine={false}
                tickMargin={8}
                tick={{ fontSize: 11 }}
                allowDecimals={false}
                width={28}
              />
              <ChartTooltip content={<ChartTooltipContent />} />
              <Bar
                dataKey="count"
                fill="var(--color-count)"
                radius={[4, 4, 0, 0]}
                barSize={10}
              />
            </BarChart>
          </ChartContainer>
        ) : (
          <div className="h-[200px] flex items-center justify-center text-muted-foreground text-sm border rounded-md border-dashed">
            No event data available
          </div>
        )}
      </div>

      {/* Events table */}
      {events.length > 0 ? (
        <div className="rounded-md border overflow-hidden">
          <Table>
            <TableHeader className="bg-muted/50 sticky top-0">
              <TableRow>
                <TableHead className="w-[130px]">Timestamp</TableHead>
                <TableHead className="w-[70px]">Level</TableHead>
                <TableHead className="w-[110px]">Component</TableHead>
                <TableHead className="w-[200px]">Message</TableHead>
                <TableHead>Data</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {events.map((event, i) => (
                <TableRow key={event.id ?? i}>
                  <TableCell className="text-xs text-muted-foreground whitespace-nowrap">
                    <FormatDate date={event.created_at} />
                  </TableCell>
                  <TableCell>
                    {event.level === "Error" ? (
                      <Badge variant="status" state="TERMINATED" size="sm">
                        {event.level}
                      </Badge>
                    ) : event.level === "Warn" ? (
                      <Badge variant="status" state="OFFERED" size="sm">
                        {event.level}
                      </Badge>
                    ) : event.level === "Info" ? (
                      <Badge variant="status" state="STARTED" size="sm">
                        {event.level}
                      </Badge>
                    ) : (
                      <Badge variant="info" size="sm">
                        {event.level ?? "—"}
                      </Badge>
                    )}
                  </TableCell>
                  <TableCell>
                    <span className="font-mono text-xs text-muted-foreground">
                      {event.component ?? "—"}
                    </span>
                  </TableCell>
                  <TableCell className="text-xs max-w-[200px]">
                    {event.message ?? (
                      <span className="italic text-muted-foreground">—</span>
                    )}
                  </TableCell>
                  <TableCell>
                    {event.data &&
                    typeof event.data === "object" &&
                    Object.keys(event.data).length > 0 ? (
                      <EventData data={event.data as Record<string, unknown>} />
                    ) : (
                      <span className="text-muted-foreground italic text-xs">
                        —
                      </span>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      ) : (
        <div className="text-muted-foreground p-8 text-center border rounded-md border-dashed">
          No events available.
        </div>
      )}
    </TabsContent>
  );
}
