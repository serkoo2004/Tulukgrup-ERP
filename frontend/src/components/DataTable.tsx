import {
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
  type ColumnDef,
  type SortingState
} from "@tanstack/react-table";
import { useState } from "react";
import { ArrowDownUp } from "lucide-react";

type DataTableProps<TData> = {
  data: TData[];
  columns: ColumnDef<TData, any>[];
  emptyText?: string;
  onRowClick?: (row: TData) => void;
};

export function DataTable<TData>({ data, columns, emptyText = "Kayıt bulunamadı.", onRowClick }: DataTableProps<TData>) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const table = useReactTable({
    data,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel()
  });

  return (
    <div className="scrollbar-thin overflow-x-auto">
      <table className="w-full min-w-[760px] border-separate border-spacing-0 text-left text-sm">
        <thead>
          {table.getHeaderGroups().map((headerGroup) => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <th
                  key={header.id}
                  className="sticky top-0 border-b border-border bg-muted px-3 py-2 text-xs font-semibold text-muted-foreground"
                >
                  {header.isPlaceholder ? null : (
                    <button
                      className="inline-flex items-center gap-1 text-left"
                      onClick={header.column.getToggleSortingHandler()}
                      type="button"
                    >
                      {flexRender(header.column.columnDef.header, header.getContext())}
                      {header.column.getCanSort() && <ArrowDownUp size={12} aria-hidden="true" />}
                    </button>
                  )}
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.length === 0 ? (
            <tr>
              <td className="px-3 py-8 text-center text-sm text-muted-foreground" colSpan={columns.length}>
                {emptyText}
              </td>
            </tr>
          ) : (
            table.getRowModel().rows.map((row) => (
              <tr
                key={row.id}
                className={onRowClick ? "cursor-pointer hover:bg-muted/50" : "hover:bg-muted/50"}
                onClick={() => onRowClick?.(row.original)}
              >
                {row.getVisibleCells().map((cell) => (
                  <td key={cell.id} className="border-b border-border px-3 py-2 align-middle">
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}
