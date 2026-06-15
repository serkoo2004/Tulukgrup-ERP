import { cn } from "../../lib/utils";

type PanelProps = React.HTMLAttributes<HTMLDivElement> & {
  title?: string;
  description?: string;
  action?: React.ReactNode;
};

export function Panel({ title, description, action, className, children, ...props }: PanelProps) {
  return (
    <section
      className={cn("rounded-lg border border-border bg-white shadow-panel", className)}
      {...props}
    >
      {(title || description || action) && (
        <div className="flex min-h-14 flex-wrap items-center justify-between gap-3 border-b border-border px-4 py-3">
          <div>
            {title && <h2 className="text-sm font-semibold text-foreground">{title}</h2>}
            {description && <p className="mt-1 text-xs text-muted-foreground">{description}</p>}
          </div>
          {action}
        </div>
      )}
      <div className="p-4">{children}</div>
    </section>
  );
}
