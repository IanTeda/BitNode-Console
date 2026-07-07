import { createFileRoute } from "@tanstack/react-router";
import { useState, useRef, useEffect } from "react";
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { format } from "date-fns";
import { CalendarIcon } from "lucide-react";
import logger from "@/lib/logger";
import { journalsClient } from "@/lib/rpc/journals";
import { useJournalsQuery, PageDirection } from "@/queries/journals";
import type { JournalsEntry } from "@/lib/generated_protos/bitnode_console/journals/journals";
import { Priority } from "@/lib/generated_protos/bitnode_console/journals/journals";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { cn } from "@/lib/utils";

const log = logger.getSubLogger({ name: "LogsRoute" });

const PRIORITY_LABEL: Record<Priority, string> = {
  [Priority.UNSPECIFIED]: "—",
  [Priority.EMERGENCY]: "EMERG",
  [Priority.ALERT]: "ALERT",
  [Priority.CRITICAL]: "CRIT",
  [Priority.ERROR]: "ERROR",
  [Priority.WARNING]: "WARN",
  [Priority.NOTICE]: "NOTICE",
  [Priority.INFO]: "INFO",
  [Priority.DEBUG]: "DEBUG",
};

function formatTimestamp(timestampUs: string): string {
  const ms = Number(BigInt(timestampUs) / 1000n);
  return new Date(ms).toLocaleString();
}

function dateToTimestampUs(date: Date | undefined, endOfDay = false): string | undefined {
  if (!date) return undefined;
  const d = new Date(date);
  if (endOfDay) {
    d.setHours(23, 59, 59, 999);
  } else {
    d.setHours(0, 0, 0, 0);
  }
  return String(BigInt(d.getTime()) * 1000n);
}

// Max entries kept in memory during a live-tail session.
const FOLLOW_MAX_ENTRIES = 500;

function useJournalFollow() {
  const [isFollowing, setIsFollowing] = useState(false);
  const [entries, setEntries] = useState<JournalsEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  function start(priority?: Priority) {
    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;

    setEntries([]);
    setError(null);
    setIsFollowing(true);

    (async () => {
      try {
        const call = journalsClient().followJournals(
          { priority: priority ?? Priority.UNSPECIFIED },
          { abort: controller.signal },
        );
        for await (const entry of call.responses) {
          setEntries((prev) => [entry, ...prev].slice(0, FOLLOW_MAX_ENTRIES));
        }
      } catch (e) {
        if (!controller.signal.aborted) {
          setError(String(e));
        }
      } finally {
        setIsFollowing(false);
      }
    })();
  }

  function stop() {
    abortRef.current?.abort();
    abortRef.current = null;
  }

  // Abort the stream if the component unmounts while following.
  useEffect(() => () => abortRef.current?.abort(), []);

  return { isFollowing, entries, error, start, stop };
}

const columnHelper = createColumnHelper<JournalsEntry>();

const columns = [
  columnHelper.accessor("timestampUs", {
    header: "Timestamp",
    cell: (info) => formatTimestamp(info.getValue()),
  }),
  columnHelper.accessor("priority", {
    header: "Priority",
    cell: (info) => PRIORITY_LABEL[info.getValue()] ?? info.getValue(),
  }),
  columnHelper.accessor("unit", {
    header: "Unit",
  }),
  columnHelper.accessor("message", {
    header: "Message",
  }),
];

export const Route = createFileRoute("/_restricted/logs")({
  component: RouteComponent,
});

const PAGE_SIZE_OPTIONS = [25, 50, 100, 200] as const;
type PageSizeOption = (typeof PAGE_SIZE_OPTIONS)[number];

const PRIORITY_OPTIONS = [
  { value: Priority.UNSPECIFIED, label: "All priorities" },
  { value: Priority.EMERGENCY, label: "Emergency" },
  { value: Priority.ALERT, label: "Alert" },
  { value: Priority.CRITICAL, label: "Critical" },
  { value: Priority.ERROR, label: "Error" },
  { value: Priority.WARNING, label: "Warning" },
  { value: Priority.NOTICE, label: "Notice" },
  { value: Priority.INFO, label: "Info" },
  { value: Priority.DEBUG, label: "Debug" },
] as const;

function RouteComponent() {
  log.info("Logs page rendered");

  const [pageToken, setPageToken] = useState<string | undefined>(undefined);
  const [tokenStack, setTokenStack] = useState<string[]>([]);
  const [reversed, setReversed] = useState(false);
  const [pageSize, setPageSize] = useState<PageSizeOption>(50);
  const [draftPriority, setDraftPriority] = useState<Priority>(Priority.UNSPECIFIED);
  const [draftDateFrom, setDraftDateFrom] = useState<Date | undefined>(undefined);
  const [draftDateTo, setDraftDateTo] = useState<Date | undefined>(undefined);

  const [appliedPriority, setAppliedPriority] = useState<Priority>(Priority.UNSPECIFIED);
  const [appliedDateFrom, setAppliedDateFrom] = useState<Date | undefined>(undefined);
  const [appliedDateTo, setAppliedDateTo] = useState<Date | undefined>(undefined);

  const follow = useJournalFollow();

  function resetPagination() {
    setPageToken(undefined);
    setTokenStack([]);
  }

  const { data, isPending, isError, error } = useJournalsQuery({
    pageSize,
    pageToken,
    pageDirection: reversed ? PageDirection.BACKWARD : PageDirection.FORWARD,
    priority: appliedPriority !== Priority.UNSPECIFIED ? appliedPriority : undefined,
    timestampFromUs: dateToTimestampUs(appliedDateFrom),
    timestampToUs: dateToTimestampUs(appliedDateTo, true),
  });

  const displayEntries = follow.isFollowing ? follow.entries : (data?.entries ?? []);

  const table = useReactTable({
    data: displayEntries,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  const hasPrev = tokenStack.length > 0;
  const hasNext = !!data?.pagination?.pageTokenNext;

  function handleNext() {
    const next = data?.pagination?.pageTokenNext;
    if (!next) return;
    setTokenStack((s) => [...s, pageToken ?? ""]);
    setPageToken(next);
  }

  function handlePrev() {
    const stack = [...tokenStack];
    const prev = stack.pop();
    setTokenStack(stack);
    setPageToken(prev === "" ? undefined : prev);
  }

  function handleReverse() {
    setReversed((r) => !r);
    resetPagination();
  }

  function handlePageSize(e: React.ChangeEvent<HTMLSelectElement>) {
    setPageSize(Number(e.target.value) as PageSizeOption);
    resetPagination();
  }

  function handlePriority(e: React.ChangeEvent<HTMLSelectElement>) {
    setDraftPriority(Number(e.target.value) as Priority);
  }

  function handleDateFrom(date: Date | undefined) {
    setDraftDateFrom(date);
  }

  function handleDateTo(date: Date | undefined) {
    setDraftDateTo(date);
  }

  function handleApply() {
    setAppliedPriority(draftPriority);
    setAppliedDateFrom(draftDateFrom);
    setAppliedDateTo(draftDateTo);
    resetPagination();
  }

  function handleClearFilters() {
    setDraftPriority(Priority.UNSPECIFIED);
    setDraftDateFrom(undefined);
    setDraftDateTo(undefined);
    setAppliedPriority(Priority.UNSPECIFIED);
    setAppliedDateFrom(undefined);
    setAppliedDateTo(undefined);
    resetPagination();
  }

  const isDirty =
    draftPriority !== appliedPriority ||
    draftDateFrom?.getTime() !== appliedDateFrom?.getTime() ||
    draftDateTo?.getTime() !== appliedDateTo?.getTime();

  const hasActiveFilters =
    appliedPriority !== Priority.UNSPECIFIED ||
    appliedDateFrom !== undefined ||
    appliedDateTo !== undefined;

  if (isPending && !follow.isFollowing) return <div>Loading logs...</div>;
  if (isError && !follow.isFollowing) return <div>Error loading logs: {String(error)}</div>;

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-wrap items-end gap-2 py-2 border-b">
        <div className="flex flex-col gap-1">
          <label className="text-xs text-muted-foreground">Priority</label>
          <select
            value={draftPriority}
            onChange={handlePriority}
            className="px-2 py-1 text-sm border rounded bg-background hover:bg-muted/50"
          >
            {PRIORITY_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>
        <div className="flex flex-col gap-1">
          <label className="text-xs text-muted-foreground">From</label>
          <Popover>
            <PopoverTrigger asChild>
              <Button
                variant="outline"
                data-empty={!draftDateFrom}
                className={cn(
                  "w-[160px] justify-start text-left font-normal",
                  "data-[empty=true]:text-muted-foreground"
                )}
              >
                <CalendarIcon data-icon="inline-start" />
                {draftDateFrom ? format(draftDateFrom, "dd MMM yyyy") : "Pick a date"}
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-auto p-0" align="start">
              <Calendar
                mode="single"
                selected={draftDateFrom}
                onSelect={handleDateFrom}
                disabled={(date) => (draftDateTo ? date > draftDateTo : false)}
              />
            </PopoverContent>
          </Popover>
        </div>
        <div className="flex flex-col gap-1">
          <label className="text-xs text-muted-foreground">To</label>
          <Popover>
            <PopoverTrigger asChild>
              <Button
                variant="outline"
                data-empty={!draftDateTo}
                className={cn(
                  "w-[160px] justify-start text-left font-normal",
                  "data-[empty=true]:text-muted-foreground"
                )}
              >
                <CalendarIcon data-icon="inline-start" />
                {draftDateTo ? format(draftDateTo, "dd MMM yyyy") : "Pick a date"}
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-auto p-0" align="start">
              <Calendar
                mode="single"
                selected={draftDateTo}
                onSelect={handleDateTo}
                disabled={(date) => (draftDateFrom ? date < draftDateFrom : false)}
              />
            </PopoverContent>
          </Popover>
        </div>
        <div className="flex gap-2 self-end">
          {isDirty && !follow.isFollowing && (
            <Button size="sm" onClick={handleApply}>
              Apply
            </Button>
          )}
          {hasActiveFilters && !follow.isFollowing && (
            <Button size="sm" variant="outline" onClick={handleClearFilters}>
              Clear
            </Button>
          )}
          {follow.isFollowing ? (
            <Button size="sm" variant="destructive" onClick={follow.stop}>
              Stop Following
            </Button>
          ) : (
            <Button size="sm" variant="outline" onClick={() => follow.start(draftPriority)}>
              Tail Follow
            </Button>
          )}
        </div>
      </div>
      {follow.isFollowing && (
        <div className="flex items-center gap-2 py-1 text-sm text-muted-foreground">
          <span className="inline-block h-2 w-2 rounded-full bg-green-500 animate-pulse" />
          Following — {follow.entries.length} entries received
        </div>
      )}
      {follow.error && (
        <div className="text-sm text-destructive py-1">Stream error: {follow.error}</div>
      )}
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id} className="border-b">
                {headerGroup.headers.map((header) => (
                  <th
                    key={header.id}
                    className="px-3 py-2 text-left font-medium text-muted-foreground"
                  >
                    {flexRender(header.column.columnDef.header, header.getContext())}
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows.map((row) => (
              <tr key={row.id} className="border-b last:border-0 hover:bg-muted/50">
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id} className="px-3 py-2 align-top">
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {!follow.isFollowing && (
        <div className="flex items-center justify-between py-2">
          <div className="flex items-center gap-2">
            <button
              onClick={handleReverse}
              className="px-3 py-1 text-sm border rounded hover:bg-muted/50"
            >
              {reversed ? "Oldest first" : "Newest first"}
            </button>
            <select
              value={pageSize}
              onChange={handlePageSize}
              className="px-2 py-1 text-sm border rounded bg-background hover:bg-muted/50"
            >
              {PAGE_SIZE_OPTIONS.map((n) => (
                <option key={n} value={n}>
                  {n} per page
                </option>
              ))}
            </select>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={handlePrev}
              disabled={!hasPrev}
              className="px-3 py-1 text-sm border rounded disabled:opacity-40 hover:bg-muted/50 disabled:cursor-not-allowed"
            >
              Previous
            </button>
            <button
              onClick={handleNext}
              disabled={!hasNext}
              className="px-3 py-1 text-sm border rounded disabled:opacity-40 hover:bg-muted/50 disabled:cursor-not-allowed"
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
