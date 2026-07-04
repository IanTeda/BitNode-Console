import { createFileRoute } from "@tanstack/react-router";
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import logger from "@/lib/logger";
import { useJournalsQuery } from "@/queries/journals";
import type { JournalsEntry } from "@/lib/generated_protos/bitnode_console/journals/journals";
import { Priority } from "@/lib/generated_protos/bitnode_console/journals/journals";

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

function RouteComponent() {
  log.info("Logs page rendered");

  const { data, isPending, isError, error } = useJournalsQuery();

  const table = useReactTable({
    data: data?.entries ?? [],
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  if (isPending) return <div>Loading logs...</div>;
  if (isError) return <div>Error loading logs: {String(error)}</div>;

  return (
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
  );
}
