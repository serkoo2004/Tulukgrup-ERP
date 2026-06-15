import { createColumnHelper, type ColumnDef } from "@tanstack/react-table";
import { useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle, Boxes, ClipboardCheck, PackagePlus, RefreshCw, Send, ShoppingCart } from "lucide-react";
import type { Dispatch, ReactNode, SetStateAction } from "react";
import { useMemo, useState } from "react";
import { appAlert, appConfirm, appPrompt } from "../components/AppDialog";
import { DataTable } from "../components/DataTable";
import { ErrorBlock, LoadingBlock } from "../components/LoadState";
import { MetricCard } from "../components/MetricCard";
import { StatusBadge } from "../components/StatusBadge";
import { Button } from "../components/ui/Button";
import { Input } from "../components/ui/Input";
import { Panel } from "../components/ui/Panel";
import { api } from "../lib/api";
import type {
  InventoryAlertRow,
  InventoryCountItemRow,
  InventoryCountRow,
  DepartmentRow,
  InventoryLotRow,
  InventoryMovementRow,
  InventoryNotificationRow,
  InventoryProductRow,
  InventoryPurchaseRequestRow,
  InventoryPurchaseSuggestionRow,
  InventoryShipmentItemRow,
  InventoryShipmentRow,
  Vehicle
} from "../lib/types";
import { formatDate, formatDateTime, formatMoney, formatQuantity, formatQuantityInput, humanize } from "../lib/utils";

const productColumn = createColumnHelper<InventoryProductRow>();
const movementColumn = createColumnHelper<InventoryMovementRow>();
const lotColumn = createColumnHelper<InventoryLotRow>();
const shipmentColumn = createColumnHelper<InventoryShipmentRow>();
const shipmentItemColumn = createColumnHelper<InventoryShipmentItemRow>();
const countColumn = createColumnHelper<InventoryCountRow>();
const countItemColumn = createColumnHelper<InventoryCountItemRow>();
const alertColumn = createColumnHelper<InventoryAlertRow>();
const suggestionColumn = createColumnHelper<InventoryPurchaseSuggestionRow>();
const purchaseRequestColumn = createColumnHelper<InventoryPurchaseRequestRow>();
const inventoryNotificationColumn = createColumnHelper<InventoryNotificationRow>();

const tabs = [
  { key: "products", label: "Ürünler" },
  { key: "movements", label: "Hareketler" },
  { key: "lots", label: "Lot / SKT" },
  { key: "shipments", label: "Sevkiyat" },
  { key: "counts", label: "Sayım" },
  { key: "alerts", label: "Kritik Stok" },
  { key: "suggestions", label: "Satın Alma" },
  { key: "requests", label: "Talepler" }
];

const units = ["adet", "koli", "paket", "kutu", "kg", "litre", "metre"];
const movementTypes = [
  { value: "warehouse_in", label: "Depo Girişi" },
  { value: "warehouse_out", label: "Depo Çıkışı" },
  { value: "branch_shipment", label: "Şubeye Sevk" },
  { value: "branch_return", label: "Şubeden İade" },
  { value: "count_adjustment", label: "Sayım Düzeltmesi" },
  { value: "scrap_out", label: "Hurda Çıkışı" },
  { value: "vehicle_usage", label: "Araç Kullanımı" },
  { value: "manual_in", label: "Manuel Giriş" },
  { value: "manual_out", label: "Manuel Çıkış" }
];

type ShipmentDraftItem = {
  product_id: number;
  product_name: string;
  quantity: number;
  unit: string;
  unit_multiplier: number;
  base_quantity: number;
};

type CountDraftItem = {
  product_id: number;
  product_name: string;
  counted_stock: number;
  reason?: string;
};

export function InventoryPage() {
  const [activeTab, setActiveTab] = useState("products");
  const [productValues, setProductValues] = useState<Record<string, string>>({ main_unit: "adet", package_unit: "koli" });
  const [productFilters, setProductFilters] = useState<Record<string, string>>({});
  const [editingProductId, setEditingProductId] = useState<number | null>(null);
  const [movementValues, setMovementValues] = useState<Record<string, string>>({ movement_type: "warehouse_in", unit: "adet", unit_multiplier: "1" });
  const [movementFilters, setMovementFilters] = useState<Record<string, string>>({});
  const [lotValues, setLotValues] = useState<Record<string, string>>({});
  const [shipmentValues, setShipmentValues] = useState<Record<string, string>>({ unit: "adet", unit_multiplier: "1" });
  const [countValues, setCountValues] = useState<Record<string, string>>({ count_type: "partial", count_method: "manual" });
  const [purchaseValues, setPurchaseValues] = useState<Record<string, string>>({});
  const [shipmentItems, setShipmentItems] = useState<ShipmentDraftItem[]>([]);
  const [countItems, setCountItems] = useState<CountDraftItem[]>([]);
  const [selectedShipmentId, setSelectedShipmentId] = useState<number | null>(null);
  const [selectedCountId, setSelectedCountId] = useState<number | null>(null);
  const queryClient = useQueryClient();
  const movementNeedsBranch = ["branch_shipment", "branch_return"].includes(movementValues.movement_type);
  const movementNeedsVehicle = movementValues.movement_type === "vehicle_usage";
  const productQueryString = useMemo(
    () =>
      buildQueryString({
        include_inactive: "true",
        limit: "500",
        q: productFilters.q,
        category: productFilters.category,
        low_stock: productFilters.low_stock
      }),
    [productFilters]
  );
  const movementQueryString = useMemo(
    () =>
      buildQueryString({
        limit: "500",
        product_id: movementFilters.product_id,
        movement_type: movementFilters.movement_type,
        branch_id: movementFilters.branch_id,
        vehicle_id: movementFilters.vehicle_id,
        date_from: movementFilters.date_from,
        date_to: movementFilters.date_to
      }),
    [movementFilters]
  );

  const [
    dashboardQuery,
    productsQuery,
    movementsQuery,
    lotsQuery,
    shipmentsQuery,
    countsQuery,
    alertsQuery,
    suggestionsQuery,
    purchaseRequestsQuery,
    departmentsQuery,
    vehiclesQuery
  ] =
    useQueries({
      queries: [
        { queryKey: ["inventory-dashboard"], queryFn: api.inventoryDashboard },
        { queryKey: ["inventory-products", productQueryString], queryFn: () => api.inventoryProducts(productQueryString) },
        { queryKey: ["inventory-movements", movementQueryString], queryFn: () => api.inventoryMovements(movementQueryString) },
        { queryKey: ["inventory-lots"], queryFn: () => api.inventoryLots("limit=200") },
        { queryKey: ["inventory-shipments"], queryFn: api.inventoryShipments },
        { queryKey: ["inventory-counts"], queryFn: api.inventoryCounts },
        { queryKey: ["inventory-alerts"], queryFn: api.inventoryAlerts },
        { queryKey: ["inventory-suggestions"], queryFn: api.inventoryPurchaseSuggestions },
        { queryKey: ["inventory-purchase-requests"], queryFn: api.inventoryPurchaseRequests },
        { queryKey: ["departments"], queryFn: api.departments },
        { queryKey: ["vehicles", "inventory-select"], queryFn: () => api.vehicles("status=active&limit=500") }
      ]
    });

  const products = productsQuery.data ?? [];
  const departments = departmentsQuery.data ?? [];
  const vehicles = vehiclesQuery.data ?? [];
  const productCategoryOptions = useMemo(
    () => Array.from(new Set(products.map((product) => product.category).filter(Boolean) as string[])).sort((a, b) => a.localeCompare(b, "tr")),
    [products]
  );
  const shipmentDetailQuery = useQuery({
    queryKey: ["inventory-shipment", selectedShipmentId],
    queryFn: () => api.inventoryShipment(selectedShipmentId as number),
    enabled: activeTab === "shipments" && selectedShipmentId !== null
  });
  const countDetailQuery = useQuery({
    queryKey: ["inventory-count", selectedCountId],
    queryFn: () => api.inventoryCount(selectedCountId as number),
    enabled: activeTab === "counts" && selectedCountId !== null
  });

  const refreshInventory = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ["inventory-dashboard"] }),
      queryClient.invalidateQueries({ queryKey: ["inventory-products"] }),
      queryClient.invalidateQueries({ queryKey: ["inventory-movements"] }),
      queryClient.invalidateQueries({ queryKey: ["inventory-lots"] }),
      queryClient.invalidateQueries({ queryKey: ["inventory-shipments"] }),
      queryClient.invalidateQueries({ queryKey: ["inventory-counts"] }),
      queryClient.invalidateQueries({ queryKey: ["inventory-alerts"] }),
      queryClient.invalidateQueries({ queryKey: ["inventory-suggestions"] }),
      queryClient.invalidateQueries({ queryKey: ["inventory-purchase-requests"] }),
      queryClient.invalidateQueries({ queryKey: ["dashboard"] })
    ]);
  };

  const productMutation = useMutation({
    mutationFn: () =>
      editingProductId
        ? api.updateInventoryProduct(editingProductId, buildProductPayload(productValues))
        : api.createInventoryProduct(buildProductPayload(productValues)),
    onSuccess: async () => {
      await refreshInventory();
      setEditingProductId(null);
      setProductValues({ main_unit: "adet", package_unit: "koli" });
    }
  });
  const deleteProductMutation = useMutation({
    mutationFn: (id: number) => api.deleteInventoryProduct(id),
    onSuccess: refreshInventory
  });
  const movementMutation = useMutation({
    mutationFn: () => api.createInventoryMovement(buildMovementPayload(movementValues)),
    onSuccess: async () => {
      await refreshInventory();
      setMovementValues({ movement_type: "warehouse_in", unit: "adet", unit_multiplier: "1" });
    }
  });
  const cancelMovementMutation = useMutation({
    mutationFn: ({ id, reason }: { id: number; reason: string }) => api.cancelInventoryMovement(id, reason),
    onSuccess: refreshInventory
  });
  const lotMutation = useMutation({
    mutationFn: () => api.createInventoryLot(buildLotPayload(lotValues)),
    onSuccess: async () => {
      await refreshInventory();
      setLotValues({});
    }
  });
  const shipmentMutation = useMutation({
    mutationFn: () => api.createInventoryShipment(buildShipmentPayload(shipmentValues, shipmentItems)),
    onSuccess: async () => {
      await refreshInventory();
      setShipmentValues({ unit: "adet", unit_multiplier: "1" });
      setShipmentItems([]);
    }
  });
  const approveShipmentMutation = useMutation({
    mutationFn: (id: number) => api.approveInventoryShipment(id),
    onSuccess: refreshInventory
  });
  const countMutation = useMutation({
    mutationFn: () => api.createInventoryCount(buildCountPayload(countValues, countItems)),
    onSuccess: async () => {
      await refreshInventory();
      setCountValues({ count_type: "partial", count_method: "manual" });
      setCountItems([]);
    }
  });
  const completeCountMutation = useMutation({
    mutationFn: (id: number) => api.completeInventoryCount(id),
    onSuccess: refreshInventory
  });
  const purchaseRequestMutation = useMutation({
    mutationFn: () => api.createInventoryPurchaseRequest(buildPurchaseRequestPayload(purchaseValues)),
    onSuccess: async () => {
      await refreshInventory();
      setPurchaseValues({});
    }
  });
  const updatePurchaseRequestMutation = useMutation({
    mutationFn: ({ id, payload }: { id: number; payload: Record<string, unknown> }) => api.updateInventoryPurchaseRequest(id, payload),
    onSuccess: refreshInventory
  });

  const productColumns = useMemo<ColumnDef<InventoryProductRow, any>[]>(
    () => [
      productColumn.accessor("product_name", { header: "Ürün", cell: (info) => <span className="font-semibold text-primary">{info.getValue()}</span> }),
      productColumn.accessor("category", { header: "Kategori", cell: (info) => info.getValue() ?? "-" }),
      productColumn.accessor("brand", { header: "Marka", cell: (info) => info.getValue() ?? "-" }),
      productColumn.accessor("current_stock", { header: "Mevcut", cell: (info) => formatQuantity(info.getValue(), info.row.original.main_unit) }),
      productColumn.accessor("available_stock", { header: "Kullanılabilir", cell: (info) => formatQuantity(info.getValue(), info.row.original.main_unit) }),
      productColumn.accessor("reserved_stock", { header: "Rezerve", cell: (info) => formatQuantity(info.getValue(), info.row.original.main_unit) }),
      productColumn.display({
        id: "package_breakdown",
        header: "Koli Karşılığı",
        cell: (info) => packageBreakdown(info.row.original)
      }),
      productColumn.accessor("minimum_stock", { header: "Min.", cell: (info) => formatQuantity(info.getValue()) }),
      productColumn.accessor("package_multiplier", {
        header: "Birim Çarpanı",
        cell: (info) => packageLabel(info.row.original)
      }),
      productColumn.display({
        id: "tracking_flags",
        header: "Takip",
        cell: (info) => trackingFlags(info.row.original)
      }),
      productColumn.accessor("is_active", { header: "Durum", cell: (info) => <StatusBadge value={info.getValue() ? "active" : "passive"} /> }),
      productColumn.display({
        id: "actions",
        header: "İşlem",
        cell: (info) => (
          <div className="flex flex-wrap gap-2">
            <Button size="sm" variant="secondary" onClick={() => startProductEdit(info.row.original, setEditingProductId, setProductValues)}>
              Düzenle
            </Button>
            <Button
              size="sm"
              variant="danger"
              onClick={async () => {
                if (await appConfirm(`${info.row.original.product_name} arşivlensin mi?`, {
                  title: "Ürün arşivle",
                  confirmLabel: "Arşivle",
                  tone: "danger"
                })) {
                  deleteProductMutation.mutate(info.row.original.id);
                }
              }}
              disabled={deleteProductMutation.isPending}
            >
              Arşivle
            </Button>
          </div>
        )
      })
    ],
    [deleteProductMutation]
  );
  const movementColumns = useMemo<ColumnDef<InventoryMovementRow, any>[]>(
    () => [
      movementColumn.accessor("product_name", { header: "Ürün" }),
      movementColumn.accessor("movement_type", { header: "Hareket", cell: (info) => humanize(info.getValue()) }),
      movementColumn.accessor("lot_number", { header: "Lot", cell: (info) => info.getValue() ?? "-" }),
      movementColumn.accessor("base_quantity", { header: "Miktar", cell: (info) => formatQuantity(info.getValue()) }),
      movementColumn.accessor("previous_stock", { header: "Önceki", cell: (info) => formatQuantity(info.getValue()) }),
      movementColumn.accessor("next_stock", { header: "Sonraki", cell: (info) => formatQuantity(info.getValue()) }),
      movementColumn.accessor("branch_name", { header: "Şube", cell: (info) => info.getValue() ?? "-" }),
      movementColumn.accessor("plate", { header: "Araç", cell: (info) => info.getValue() ?? "-" }),
      movementColumn.accessor("movement_at", { header: "Tarih", cell: (info) => formatDateTime(info.getValue()) }),
      movementColumn.display({
        id: "actions",
        header: "İşlem",
        cell: (info) =>
          info.row.original.cancelled_at ? (
            <StatusBadge value="cancelled" />
          ) : (
            <Button
              size="sm"
              variant="secondary"
              onClick={async () => {
                const reason = await appPrompt("İptal nedeni", {
                  title: "Stok hareketini iptal et",
                  confirmLabel: "İptal et"
                });
                if (reason) cancelMovementMutation.mutate({ id: info.row.original.id, reason });
              }}
            >
              İptal
            </Button>
          )
      })
    ],
    [cancelMovementMutation]
  );
  const recentMovementColumns = useMemo<ColumnDef<InventoryMovementRow, any>[]>(
    () => [
      movementColumn.accessor("product_name", { header: "Ürün" }),
      movementColumn.accessor("movement_type", { header: "Hareket", cell: (info) => humanize(info.getValue()) }),
      movementColumn.accessor("base_quantity", { header: "Miktar", cell: (info) => formatQuantity(info.getValue()) }),
      movementColumn.accessor("next_stock", { header: "Son Stok", cell: (info) => formatQuantity(info.getValue()) }),
      movementColumn.accessor("movement_at", { header: "Tarih", cell: (info) => formatDateTime(info.getValue()) })
    ],
    []
  );
  const lotColumns = [
    lotColumn.accessor("product_name", { header: "Ürün" }),
    lotColumn.accessor("lot_number", { header: "Lot" }),
    lotColumn.accessor("supplier", { header: "Tedarikçi", cell: (info) => info.getValue() ?? "-" }),
    lotColumn.accessor("quantity", { header: "Miktar", cell: (info) => formatQuantity(info.getValue()) }),
    lotColumn.accessor("expiry_date", { header: "SKT", cell: (info) => formatDate(info.getValue()) })
  ];
  const shipmentColumns = [
    shipmentColumn.accessor("id", { header: "Sevk", cell: (info) => `#${info.getValue()}` }),
    shipmentColumn.accessor("branch_name", { header: "Şube", cell: (info) => info.getValue() ?? "-" }),
    shipmentColumn.accessor("shipment_status", { header: "Durum", cell: (info) => <StatusBadge value={info.getValue()} /> }),
    shipmentColumn.accessor("item_count", { header: "Kalem" }),
    shipmentColumn.accessor("created_at", { header: "Tarih", cell: (info) => formatDateTime(info.getValue()) }),
    shipmentColumn.display({
      id: "approve",
      header: "Onay",
      cell: (info) =>
        info.row.original.shipment_status === "draft" ? (
          <Button size="sm" onClick={() => approveShipmentMutation.mutate(info.row.original.id)}>
            Onayla
          </Button>
        ) : (
          "-"
        )
    }),
    shipmentColumn.display({
      id: "detail",
      header: "Detay",
      cell: (info) => (
        <Button size="sm" variant="secondary" onClick={() => setSelectedShipmentId(info.row.original.id)}>
          Detay
        </Button>
      )
    })
  ];
  const countColumns = [
    countColumn.accessor("id", { header: "Sayım", cell: (info) => `#${info.getValue()}` }),
    countColumn.accessor("count_type", { header: "Tür", cell: (info) => humanize(info.getValue()) }),
    countColumn.accessor("count_status", { header: "Durum", cell: (info) => <StatusBadge value={info.getValue()} /> }),
    countColumn.accessor("item_count", { header: "Kalem" }),
    countColumn.accessor("created_at", { header: "Tarih", cell: (info) => formatDateTime(info.getValue()) }),
    countColumn.display({
      id: "complete",
      header: "Tamamla",
      cell: (info) =>
        info.row.original.count_status === "draft" ? (
          <Button size="sm" onClick={() => completeCountMutation.mutate(info.row.original.id)}>
            Tamamla
          </Button>
        ) : (
          "-"
        )
    }),
    countColumn.display({
      id: "detail",
      header: "Detay",
      cell: (info) => (
        <Button size="sm" variant="secondary" onClick={() => setSelectedCountId(info.row.original.id)}>
          Detay
        </Button>
      )
    })
  ];
  const shipmentItemColumns = [
    shipmentItemColumn.accessor("product_name", { header: "Ürün" }),
    shipmentItemColumn.accessor("quantity", { header: "Girilen", cell: (info) => formatQuantity(info.getValue(), info.row.original.unit) }),
    shipmentItemColumn.accessor("unit_multiplier", { header: "Çarpan", cell: (info) => formatQuantity(info.getValue()) }),
    shipmentItemColumn.accessor("base_quantity", { header: "Ana Miktar", cell: (info) => formatQuantity(info.getValue()) })
  ];
  const countItemColumns = [
    countItemColumn.accessor("product_name", { header: "Ürün" }),
    countItemColumn.accessor("system_stock", { header: "Sistem", cell: (info) => formatQuantity(info.getValue()) }),
    countItemColumn.accessor("counted_stock", { header: "Sayım", cell: (info) => formatQuantity(info.getValue()) }),
    countItemColumn.accessor("difference", { header: "Fark", cell: (info) => formatQuantity(info.getValue()) }),
    countItemColumn.accessor("reason", { header: "Neden", cell: (info) => info.getValue() ?? "-" })
  ];
  const alertColumns = [
    alertColumn.accessor("product_name", { header: "Ürün" }),
    alertColumn.accessor("current_stock", { header: "Mevcut", cell: (info) => formatQuantity(info.getValue()) }),
    alertColumn.accessor("minimum_stock", { header: "Minimum", cell: (info) => formatQuantity(info.getValue()) }),
    alertColumn.accessor("alert_level", { header: "Alarm", cell: (info) => <StatusBadge value={info.getValue()} /> }),
    alertColumn.accessor("suggested_order_quantity", { header: "Önerilen", cell: (info) => formatQuantity(info.getValue()) })
  ];
  const suggestionColumns = [
    suggestionColumn.accessor("product_name", { header: "Ürün" }),
    suggestionColumn.accessor("current_stock", { header: "Stok", cell: (info) => formatQuantity(info.getValue()) }),
    suggestionColumn.accessor("average_monthly_consumption", { header: "Aylık Ort.", cell: (info) => formatQuantity(info.getValue()) }),
    suggestionColumn.accessor("estimated_runout_date", { header: "Tahmini Tükenme", cell: (info) => formatDate(info.getValue()) }),
    suggestionColumn.accessor("suggested_order_quantity", { header: "Önerilen Sipariş", cell: (info) => formatQuantity(info.getValue()) }),
    suggestionColumn.display({
      id: "request",
      header: "Talep",
      cell: (info) => (
        <Button
          size="sm"
          onClick={() => {
            setPurchaseValues({
              product_id: String(info.row.original.product_id),
              requested_quantity: formatQuantityInput(info.row.original.suggested_order_quantity),
              reason: "Kritik stok / satın alma önerisi"
            });
            setActiveTab("requests");
          }}
        >
          Talebe Aktar
        </Button>
      )
    })
  ];
  const purchaseRequestColumns = [
    purchaseRequestColumn.accessor("product_name", { header: "Ürün" }),
    purchaseRequestColumn.accessor("requested_quantity", { header: "Talep", cell: (info) => formatQuantity(info.getValue()) }),
    purchaseRequestColumn.accessor("approved_quantity", { header: "Onay", cell: (info) => (info.getValue() === null ? "-" : formatQuantity(info.getValue())) }),
    purchaseRequestColumn.accessor("request_status", { header: "Durum", cell: (info) => <StatusBadge value={info.getValue()} /> }),
    purchaseRequestColumn.accessor("reason", { header: "Neden", cell: (info) => info.getValue() ?? "-" }),
    purchaseRequestColumn.accessor("created_at", { header: "Tarih", cell: (info) => formatDateTime(info.getValue()) }),
    purchaseRequestColumn.display({
      id: "actions",
      header: "İşlem",
      cell: (info) => (
        <div className="flex flex-wrap gap-2">
          {["approved", "ordered", "received", "cancelled"].map((status) => (
            <Button
              key={status}
              size="sm"
              variant="secondary"
              onClick={async () => {
                const update = await buildPurchaseRequestStatusUpdate(info.row.original, status);
                if (update) updatePurchaseRequestMutation.mutate(update);
              }}
              disabled={updatePurchaseRequestMutation.isPending}
            >
              {humanize(status)}
            </Button>
          ))}
        </div>
      )
    })
  ];
  const inventoryNotificationColumns = [
    inventoryNotificationColumn.accessor("message", { header: "Bildirim" }),
    inventoryNotificationColumn.accessor("sent_via", { header: "Kanal", cell: (info) => humanize(info.getValue()) }),
    inventoryNotificationColumn.accessor("delivery_status", { header: "Durum", cell: (info) => <StatusBadge value={info.getValue()} /> }),
    inventoryNotificationColumn.accessor("created_at", { header: "Tarih", cell: (info) => formatDateTime(info.getValue()) })
  ];

  const loading = [
    dashboardQuery,
    productsQuery,
    movementsQuery,
    lotsQuery,
    shipmentsQuery,
    countsQuery,
    alertsQuery,
    suggestionsQuery,
    purchaseRequestsQuery,
    departmentsQuery,
    vehiclesQuery
  ].some((query) => query.isLoading);
  const error = [
    dashboardQuery,
    productsQuery,
    movementsQuery,
    lotsQuery,
    shipmentsQuery,
    countsQuery,
    alertsQuery,
    suggestionsQuery,
    purchaseRequestsQuery,
    departmentsQuery,
    vehiclesQuery
  ].find((query) => query.isError)?.error;
  if (loading) return <LoadingBlock />;
  if (error) return <ErrorBlock error={error} />;
  const dashboard = dashboardQuery.data;

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold">Merkez Stok</h1>
        <p className="mt-1 text-sm text-muted-foreground">Ürün kartları, stok hareketleri, sevkiyat, sayım, lot/SKT ve satın alma önerileri.</p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-6">
        <MetricCard icon={Boxes} label="Ürün" value={dashboard?.total_products ?? 0} helper="Aktif ürün kartı" />
        <MetricCard icon={ShoppingCart} label="Stok Değeri" value={formatMoney(dashboard?.total_stock_value ?? null)} helper="Birim maliyet bazlı" />
        <MetricCard icon={AlertTriangle} label="Kritik" value={dashboard?.critical_stock_count ?? 0} helper="Minimum/kritik seviye" tone="warning" />
        <MetricCard icon={AlertTriangle} label="Tükenen" value={dashboard?.out_of_stock_count ?? 0} helper="Stok sıfır" tone="danger" />
        <MetricCard icon={Send} label="Sevkiyat" value={dashboard?.pending_shipments ?? 0} helper="Bekleyen onay" />
        <MetricCard icon={ClipboardCheck} label="SKT 30 Gün" value={dashboard?.expiring_lots_30_days ?? 0} helper="Yaklaşan lot" />
      </div>

      <div className="grid gap-4 xl:grid-cols-2">
        <Panel title="Son Stok Hareketleri">
          <DataTable data={dashboard?.recent_movements ?? []} columns={recentMovementColumns} emptyText="Son stok hareketi yok." />
        </Panel>
        <Panel title="Son Stok Bildirimleri">
          <DataTable data={dashboard?.recent_notifications ?? []} columns={inventoryNotificationColumns} emptyText="Son stok bildirimi yok." />
        </Panel>
      </div>

      <Panel
        title="Stok Modülü"
        action={
          <div className="flex flex-wrap gap-2">
            {tabs.map((tab) => (
              <Button key={tab.key} size="sm" variant={activeTab === tab.key ? "primary" : "secondary"} onClick={() => setActiveTab(tab.key)}>
                {tab.label}
              </Button>
            ))}
          </div>
        }
      >
        {activeTab === "products" && (
          <div className="space-y-4">
            <InventoryForm
              title={editingProductId ? "Ürün Kartı Düzenle" : "Ürün Kartı"}
              error={productMutation.error ?? deleteProductMutation.error}
              onSubmit={() => productMutation.mutate()}
              disabled={productMutation.isPending}
            >
              <TextField label="Ürün Adı" name="product_name" required values={productValues} setValues={setProductValues} />
              <TextField label="Kategori" name="category" values={productValues} setValues={setProductValues} />
              <TextField label="Alt Kategori" name="sub_category" values={productValues} setValues={setProductValues} />
              <TextField label="Marka" name="brand" values={productValues} setValues={setProductValues} />
              <SelectField label="Ana Birim" name="main_unit" options={units} values={productValues} setValues={setProductValues} />
              <SelectField label="Koli/Paket Birimi" name="package_unit" options={units} values={productValues} setValues={setProductValues} />
              <TextField label="Koli Çarpanı" name="package_multiplier" type="number" values={productValues} setValues={setProductValues} />
              <TextField label="Minimum" name="minimum_stock" type="number" values={productValues} setValues={setProductValues} />
              <TextField label="Maksimum" name="maximum_stock" type="number" values={productValues} setValues={setProductValues} />
              <TextField label="Kritik" name="critical_stock" type="number" values={productValues} setValues={setProductValues} />
              <TextField label="Güvenlik" name="safety_stock" type="number" values={productValues} setValues={setProductValues} />
              <TextField label="Birim Maliyet" name="unit_cost" type="number" values={productValues} setValues={setProductValues} />
              <CheckboxField label="Lot Takibi" name="is_lot_tracked" values={productValues} setValues={setProductValues} />
              <CheckboxField label="SKT Takibi" name="expiry_tracking" values={productValues} setValues={setProductValues} />
            </InventoryForm>
            {editingProductId && (
              <div className="flex justify-end">
                <Button
                  type="button"
                  variant="secondary"
                  onClick={() => {
                    setEditingProductId(null);
                    setProductValues({ main_unit: "adet", package_unit: "koli" });
                  }}
                >
                  Düzenlemeyi İptal Et
                </Button>
              </div>
            )}
            <InventoryFilterBar onClear={() => setProductFilters({})}>
              <TextField label="Ürün / kategori / marka ara" name="q" values={productFilters} setValues={setProductFilters} />
              <SelectField
                label="Kategori"
                name="category"
                options={productCategoryOptions}
                values={productFilters}
                setValues={setProductFilters}
              />
              <SelectField
                label="Stok Durumu"
                name="low_stock"
                options={[{ value: "true", label: "Kritik / düşük stok" }]}
                values={productFilters}
                setValues={setProductFilters}
              />
            </InventoryFilterBar>
            <DataTable data={products} columns={productColumns} emptyText="Ürün kartı yok." />
          </div>
        )}

        {activeTab === "movements" && (
          <div className="space-y-4">
            <InventoryForm title="Stok Hareketi" error={movementMutation.error ?? cancelMovementMutation.error} onSubmit={() => movementMutation.mutate()} disabled={movementMutation.isPending}>
              <ProductSelect products={products} values={movementValues} setValues={setMovementValues} syncUnit />
              <MovementTypeSelect values={movementValues} setValues={setMovementValues} />
              <LotSelect lots={lotsQuery.data ?? []} products={products} values={movementValues} setValues={setMovementValues} />
              <TextField label="Miktar" name="quantity" type="number" required values={movementValues} setValues={setMovementValues} />
              <InventoryUnitSelect products={products} values={movementValues} setValues={setMovementValues} />
              <TextField label="Birim Çarpanı" name="unit_multiplier" type="number" values={movementValues} setValues={setMovementValues} />
              {movementNeedsBranch && <DepartmentSelect label="Şube / Birim" departments={departments} values={movementValues} setValues={setMovementValues} />}
              {movementNeedsVehicle && <VehicleSelect vehicles={vehicles} values={movementValues} setValues={setMovementValues} />}
              <TextField label="Açıklama" name="description" values={movementValues} setValues={setMovementValues} />
              <ConversionPreview products={products} values={movementValues} />
            </InventoryForm>
            <InventoryFilterBar onClear={() => setMovementFilters({})}>
              <ProductFilterSelect products={products} values={movementFilters} setValues={setMovementFilters} />
              <SelectField label="Hareket Tipi" name="movement_type" options={movementTypes} values={movementFilters} setValues={setMovementFilters} />
              <DepartmentSelect label="Şube / Birim" departments={departments} values={movementFilters} setValues={setMovementFilters} />
              <VehicleSelect vehicles={vehicles} values={movementFilters} setValues={setMovementFilters} />
              <TextField label="Başlangıç" name="date_from" type="date" values={movementFilters} setValues={setMovementFilters} />
              <TextField label="Bitiş" name="date_to" type="date" values={movementFilters} setValues={setMovementFilters} />
            </InventoryFilterBar>
            <DataTable data={movementsQuery.data ?? []} columns={movementColumns} emptyText="Stok hareketi yok." />
          </div>
        )}

        {activeTab === "lots" && (
          <div className="space-y-4">
            <InventoryForm title="Lot / Parti" error={lotMutation.error} onSubmit={() => lotMutation.mutate()} disabled={lotMutation.isPending}>
              <ProductSelect products={products} values={lotValues} setValues={setLotValues} />
              <TextField label="Lot No" name="lot_number" required values={lotValues} setValues={setLotValues} />
              <TextField label="Üretim Tarihi" name="production_date" type="date" values={lotValues} setValues={setLotValues} />
              <TextField label="Son Kullanma" name="expiry_date" type="date" values={lotValues} setValues={setLotValues} />
              <TextField label="Tedarikçi" name="supplier" values={lotValues} setValues={setLotValues} />
              <TextField label="Miktar" name="quantity" type="number" values={lotValues} setValues={setLotValues} />
            </InventoryForm>
            <DataTable data={lotsQuery.data ?? []} columns={lotColumns} emptyText="Lot kaydı yok." />
          </div>
        )}

        {activeTab === "shipments" && (
          <div className="space-y-4">
            <InventoryForm
              title="Şube Sevkiyatı"
              error={shipmentMutation.error ?? approveShipmentMutation.error}
              onSubmit={() => shipmentMutation.mutate()}
              disabled={shipmentMutation.isPending || shipmentItems.length === 0}
              submitLabel="Sevkiyatı Kaydet"
            >
              <DepartmentSelect label="Gönderilen Şube / Birim" departments={departments} values={shipmentValues} setValues={setShipmentValues} />
              <ProductSelect products={products} values={shipmentValues} setValues={setShipmentValues} syncUnit />
              <TextField label="Miktar" name="quantity" type="number" required values={shipmentValues} setValues={setShipmentValues} />
              <InventoryUnitSelect products={products} values={shipmentValues} setValues={setShipmentValues} />
              <TextField label="Birim Çarpanı" name="unit_multiplier" type="number" values={shipmentValues} setValues={setShipmentValues} />
              <TextField label="Not" name="note" values={shipmentValues} setValues={setShipmentValues} />
              <ConversionPreview products={products} values={shipmentValues} />
              <div className="flex items-end md:col-span-4">
                <Button type="button" variant="secondary" onClick={() => addShipmentDraftItem(products, shipmentValues, setShipmentValues, setShipmentItems)}>
                  Kalem Ekle
                </Button>
              </div>
              <DraftItemsPanel
                title="Sevkiyat Kalemleri"
                emptyText="Henüz sevkiyat kalemi eklenmedi."
                items={shipmentItems.map((item, index) => ({
                  id: `${item.product_id}-${index}`,
                  label: item.product_name,
                  detail: `${formatQuantity(item.quantity, item.unit)} = ${formatQuantity(item.base_quantity, products.find((product) => product.id === item.product_id)?.main_unit ?? "adet")}`
                }))}
                onRemove={(index) => setShipmentItems((current) => current.filter((_, itemIndex) => itemIndex !== index))}
              />
            </InventoryForm>
            <DataTable data={shipmentsQuery.data ?? []} columns={shipmentColumns} emptyText="Sevkiyat yok." />
            <InventoryDetailPanel
              title={shipmentDetailQuery.data ? `Sevkiyat #${shipmentDetailQuery.data.shipment.id} Kalemleri` : "Sevkiyat Detayı"}
              loading={shipmentDetailQuery.isLoading}
              error={shipmentDetailQuery.error}
              selected={selectedShipmentId !== null}
              onClose={() => setSelectedShipmentId(null)}
            >
              <DataTable data={shipmentDetailQuery.data?.items ?? []} columns={shipmentItemColumns} emptyText="Sevkiyat kalemi yok." />
            </InventoryDetailPanel>
          </div>
        )}

        {activeTab === "counts" && (
          <div className="space-y-4">
            <InventoryForm
              title="Depo Sayımı"
              error={countMutation.error ?? completeCountMutation.error}
              onSubmit={() => countMutation.mutate()}
              disabled={countMutation.isPending || countItems.length === 0}
              submitLabel="Sayımı Kaydet"
            >
              <SelectField label="Sayım Türü" name="count_type" options={["full", "partial", "category"]} values={countValues} setValues={setCountValues} />
              <SelectField label="Yöntem" name="count_method" options={["manual", "qr"]} values={countValues} setValues={setCountValues} />
              <TextField label="Kategori" name="category" values={countValues} setValues={setCountValues} />
              <ProductSelect products={products} values={countValues} setValues={setCountValues} />
              <TextField label="Sayım Stok" name="counted_stock" type="number" required values={countValues} setValues={setCountValues} />
              <TextField label="Fark Nedeni" name="reason" values={countValues} setValues={setCountValues} />
              <div className="flex items-end md:col-span-4">
                <Button type="button" variant="secondary" onClick={() => addCountDraftItem(products, countValues, setCountValues, setCountItems)}>
                  Kalem Ekle
                </Button>
              </div>
              <DraftItemsPanel
                title="Sayım Kalemleri"
                emptyText="Henüz sayım kalemi eklenmedi."
                items={countItems.map((item, index) => ({
                  id: `${item.product_id}-${index}`,
                  label: item.product_name,
                  detail: `${formatQuantity(item.counted_stock)}${item.reason ? ` / ${item.reason}` : ""}`
                }))}
                onRemove={(index) => setCountItems((current) => current.filter((_, itemIndex) => itemIndex !== index))}
              />
            </InventoryForm>
            <DataTable data={countsQuery.data ?? []} columns={countColumns} emptyText="Sayım kaydı yok." />
            <InventoryDetailPanel
              title={countDetailQuery.data ? `Sayım #${countDetailQuery.data.count.id} Kalemleri` : "Sayım Detayı"}
              loading={countDetailQuery.isLoading}
              error={countDetailQuery.error}
              selected={selectedCountId !== null}
              onClose={() => setSelectedCountId(null)}
            >
              <DataTable data={countDetailQuery.data?.items ?? []} columns={countItemColumns} emptyText="Sayım kalemi yok." />
            </InventoryDetailPanel>
          </div>
        )}

        {activeTab === "alerts" && <DataTable data={alertsQuery.data ?? []} columns={alertColumns} emptyText="Kritik stok yok." />}
        {activeTab === "suggestions" && <DataTable data={suggestionsQuery.data ?? []} columns={suggestionColumns} emptyText="Satın alma önerisi yok." />}
        {activeTab === "requests" && (
          <div className="space-y-4">
            <InventoryForm title="Satın Alma Talebi" error={purchaseRequestMutation.error ?? updatePurchaseRequestMutation.error} onSubmit={() => purchaseRequestMutation.mutate()} disabled={purchaseRequestMutation.isPending}>
              <ProductSelect products={products} values={purchaseValues} setValues={setPurchaseValues} />
              <TextField label="Talep Miktarı" name="requested_quantity" type="number" required values={purchaseValues} setValues={setPurchaseValues} />
              <TextField label="Neden" name="reason" values={purchaseValues} setValues={setPurchaseValues} />
              <TextField label="Not" name="note" values={purchaseValues} setValues={setPurchaseValues} />
            </InventoryForm>
            <DataTable data={purchaseRequestsQuery.data ?? []} columns={purchaseRequestColumns} emptyText="Satın alma talebi yok." />
          </div>
        )}
      </Panel>
    </div>
  );
}

function InventoryForm({
  title,
  children,
  error,
  onSubmit,
  disabled = false,
  submitLabel = "Kaydet"
}: {
  title: string;
  children: ReactNode;
  error: unknown;
  onSubmit: () => void;
  disabled?: boolean;
  submitLabel?: string;
}) {
  return (
    <form
      className="rounded-lg border border-border bg-background p-4"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit();
      }}
    >
      <div className="mb-3 flex items-center gap-2 text-sm font-semibold">
        <PackagePlus size={16} />
        {title}
      </div>
      <div className="grid gap-3 md:grid-cols-4">{children}</div>
      {Boolean(error) && (
        <div className="mt-3 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {error instanceof Error ? error.message : "İşlem tamamlanamadı"}
        </div>
      )}
      <div className="mt-4 flex justify-end">
        <Button type="submit" disabled={disabled}>
          <RefreshCw size={16} />
          {submitLabel}
        </Button>
      </div>
    </form>
  );
}

function InventoryDetailPanel({
  title,
  children,
  loading,
  error,
  selected,
  onClose
}: {
  title: string;
  children: ReactNode;
  loading: boolean;
  error: unknown;
  selected: boolean;
  onClose: () => void;
}) {
  if (!selected) return null;
  return (
    <div className="rounded-lg border border-border bg-background p-4">
      <div className="mb-3 flex items-center justify-between gap-3">
        <div className="text-sm font-semibold">{title}</div>
        <Button size="sm" variant="secondary" onClick={onClose}>
          Kapat
        </Button>
      </div>
      {loading ? <LoadingBlock /> : error ? <ErrorBlock error={error} /> : children}
    </div>
  );
}

function InventoryFilterBar({ children, onClear }: { children: ReactNode; onClear: () => void }) {
  return (
    <div className="rounded-md border border-border bg-muted/20 p-3">
      <div className="grid gap-3 md:grid-cols-6">{children}</div>
      <div className="mt-3 flex justify-end">
        <Button type="button" size="sm" variant="secondary" onClick={onClear}>
          Temizle
        </Button>
      </div>
    </div>
  );
}

function DraftItemsPanel({
  title,
  emptyText,
  items,
  onRemove
}: {
  title: string;
  emptyText: string;
  items: Array<{ id: string; label: string; detail: string }>;
  onRemove: (index: number) => void;
}) {
  return (
    <div className="rounded-md border border-border bg-muted/25 p-3 md:col-span-4">
      <div className="mb-2 text-xs font-semibold text-muted-foreground">{title}</div>
      {items.length === 0 ? (
        <div className="text-sm text-muted-foreground">{emptyText}</div>
      ) : (
        <div className="space-y-2">
          {items.map((item, index) => (
            <div key={item.id} className="flex flex-wrap items-center justify-between gap-2 rounded-md border border-border bg-background px-3 py-2">
              <div>
                <div className="text-sm font-medium">{item.label}</div>
                <div className="text-xs text-muted-foreground">{item.detail}</div>
              </div>
              <Button type="button" size="sm" variant="secondary" onClick={() => onRemove(index)}>
                Sil
              </Button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function ProductFilterSelect({
  products,
  values,
  setValues
}: {
  products: InventoryProductRow[];
  values: Record<string, string>;
  setValues: Dispatch<SetStateAction<Record<string, string>>>;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">Ürün</span>
      <select
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values.product_id ?? ""}
        onChange={(event) => setValues((current) => ({ ...current, product_id: event.target.value }))}
      >
        <option value="">Tüm ürünler</option>
        {products.map((product) => (
          <option key={product.id} value={product.id}>
            {product.product_name}
          </option>
        ))}
      </select>
    </label>
  );
}

function TextField({ label, name, type = "text", required, values, setValues }: FieldProps) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">{label}</span>
      <Input required={required} type={type} value={values[name] ?? ""} onChange={(event) => setValues((current) => ({ ...current, [name]: event.target.value }))} />
    </label>
  );
}

function CheckboxField({ label, name, values, setValues }: FieldProps) {
  return (
    <label className="flex h-10 items-center gap-2 rounded-md border border-input bg-white px-3 text-sm">
      <input
        type="checkbox"
        checked={values[name] === "true"}
        onChange={(event) => setValues((current) => ({ ...current, [name]: event.target.checked ? "true" : "false" }))}
      />
      <span className="text-xs font-medium text-muted-foreground">{label}</span>
    </label>
  );
}

function SelectField({
  label,
  name,
  options,
  values,
  setValues
}: {
  label: string;
  name: string;
  options: Array<string | { value: string; label: string }>;
  values: Record<string, string>;
  setValues: Dispatch<SetStateAction<Record<string, string>>>;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">{label}</span>
      <select
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values[name] ?? ""}
        onChange={(event) => setValues((current) => ({ ...current, [name]: event.target.value }))}
      >
        <option value="">Seçiniz</option>
        {options.map((option) => {
          const value = typeof option === "string" ? option : option.value;
          const labelText = typeof option === "string" ? humanize(option) : option.label;
          return (
            <option key={value} value={value}>
              {labelText}
            </option>
          );
        })}
      </select>
    </label>
  );
}

function MovementTypeSelect({
  values,
  setValues
}: {
  values: Record<string, string>;
  setValues: Dispatch<SetStateAction<Record<string, string>>>;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">Hareket</span>
      <select
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values.movement_type ?? ""}
        onChange={(event) => {
          const movementType = event.target.value;
          setValues((current) => ({
            ...current,
            movement_type: movementType,
            branch_id: ["branch_shipment", "branch_return"].includes(movementType) ? current.branch_id ?? "" : "",
            vehicle_id: movementType === "vehicle_usage" ? current.vehicle_id ?? "" : ""
          }));
        }}
      >
        {movementTypes.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

function ProductSelect({
  products,
  values,
  setValues,
  syncUnit = false
}: {
  products: InventoryProductRow[];
  values: Record<string, string>;
  setValues: Dispatch<SetStateAction<Record<string, string>>>;
  syncUnit?: boolean;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">Ürün</span>
      <select
        required
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values.product_id ?? ""}
        onChange={(event) => {
          const product = products.find((item) => String(item.id) === event.target.value);
          setValues((current) => ({
            ...current,
            product_id: event.target.value,
            lot_id: "",
            ...(syncUnit && product ? { unit: product.main_unit, unit_multiplier: "1" } : {})
          }));
        }}
      >
        <option value="">Ürün seç</option>
        {products.map((product) => (
          <option key={product.id} value={product.id}>
            {product.product_name}
          </option>
        ))}
      </select>
    </label>
  );
}

function LotSelect({
  lots,
  products,
  values,
  setValues
}: {
  lots: InventoryLotRow[];
  products: InventoryProductRow[];
  values: Record<string, string>;
  setValues: Dispatch<SetStateAction<Record<string, string>>>;
}) {
  const product = products.find((item) => String(item.id) === values.product_id);
  const productLots = product
    ? lots.filter((lot) => lot.product_id === product.id && lot.is_active)
    : [];

  if (!product || (!product.is_lot_tracked && productLots.length === 0)) return null;

  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">Lot / Parti</span>
      <select
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values.lot_id ?? ""}
        onChange={(event) => setValues((current) => ({ ...current, lot_id: event.target.value }))}
      >
        <option value="">Lot seçme</option>
        {productLots.map((lot) => (
          <option key={lot.id} value={lot.id}>
            {lotOptionLabel(lot, product.main_unit)}
          </option>
        ))}
      </select>
    </label>
  );
}

function DepartmentSelect({
  label,
  departments,
  values,
  setValues
}: {
  label: string;
  departments: DepartmentRow[];
  values: Record<string, string>;
  setValues: Dispatch<SetStateAction<Record<string, string>>>;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">{label}</span>
      <select
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values.branch_id ?? ""}
        onChange={(event) => setValues((current) => ({ ...current, branch_id: event.target.value }))}
      >
        <option value="">Seçiniz</option>
        {departments
          .filter((department) => department.is_active)
          .map((department) => (
            <option key={department.id} value={department.id}>
              {department.name}
            </option>
          ))}
      </select>
    </label>
  );
}

function VehicleSelect({
  vehicles,
  values,
  setValues
}: {
  vehicles: Vehicle[];
  values: Record<string, string>;
  setValues: Dispatch<SetStateAction<Record<string, string>>>;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">Araç</span>
      <select
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values.vehicle_id ?? ""}
        onChange={(event) => setValues((current) => ({ ...current, vehicle_id: event.target.value }))}
      >
        <option value="">Seçiniz</option>
        {vehicles.map((vehicle) => (
          <option key={vehicle.id} value={vehicle.id}>
            {vehicle.plate} / {vehicle.brand} {vehicle.model}
          </option>
        ))}
      </select>
    </label>
  );
}

function InventoryUnitSelect({
  products,
  values,
  setValues
}: {
  products: InventoryProductRow[];
  values: Record<string, string>;
  setValues: Dispatch<SetStateAction<Record<string, string>>>;
}) {
  const product = products.find((item) => String(item.id) === values.product_id);
  const options = product
    ? Array.from(new Set([product.main_unit, product.package_unit, ...units].filter(Boolean) as string[]))
    : units;

  return (
    <label className="block">
      <span className="mb-1 block text-xs font-medium text-muted-foreground">Birim</span>
      <select
        className="h-10 w-full rounded-md border border-input bg-white px-3 text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/15"
        value={values.unit ?? ""}
        onChange={(event) => {
          const unit = event.target.value;
          setValues((current) => ({
            ...current,
            unit,
            unit_multiplier: unitMultiplierForProduct(product, unit)
          }));
        }}
      >
        <option value="">Seçiniz</option>
        {options.map((option) => (
          <option key={option} value={option}>
            {humanize(option)}
          </option>
        ))}
      </select>
    </label>
  );
}

function ConversionPreview({ products, values }: { products: InventoryProductRow[]; values: Record<string, string> }) {
  const product = products.find((item) => String(item.id) === values.product_id);
  const quantity = Number(values.quantity);
  const multiplier = Number(values.unit_multiplier || 1);
  if (!product || !Number.isFinite(quantity) || quantity <= 0 || !Number.isFinite(multiplier) || multiplier <= 0) return null;

  const baseQuantity = quantity * multiplier;
  return (
    <div className="rounded-md border border-primary/20 bg-primary/5 px-3 py-2 text-sm text-primary md:col-span-4">
      {formatQuantity(quantity, values.unit || product.main_unit)} = {formatQuantity(baseQuantity, product.main_unit)}
    </div>
  );
}

type FieldProps = {
  label: string;
  name: string;
  type?: string;
  required?: boolean;
  values: Record<string, string>;
  setValues: Dispatch<SetStateAction<Record<string, string>>>;
};

function packageLabel(row: InventoryProductRow) {
  if (!row.package_multiplier || !row.package_unit) return "-";
  return `1 ${row.package_unit} = ${formatQuantity(row.package_multiplier, row.main_unit)}`;
}

function packageBreakdown(row: InventoryProductRow) {
  if (!row.package_multiplier || !row.package_unit) return "-";
  const stock = Number(row.current_stock);
  const multiplier = Number(row.package_multiplier);
  if (!Number.isFinite(stock) || !Number.isFinite(multiplier) || multiplier <= 0) return "-";

  const packageCount = Math.trunc(stock / multiplier);
  const remainder = stock - packageCount * multiplier;
  if (remainder <= 0) return formatQuantity(packageCount, row.package_unit);
  if (packageCount <= 0) return formatQuantity(remainder, row.main_unit);
  return `${formatQuantity(packageCount, row.package_unit)} + ${formatQuantity(remainder, row.main_unit)}`;
}

function trackingFlags(row: InventoryProductRow) {
  const flags = [];
  if (row.is_lot_tracked) flags.push("Lot");
  if (row.expiry_tracking) flags.push("SKT");
  return flags.length > 0 ? flags.join(" / ") : "-";
}

function unitMultiplierForProduct(product: InventoryProductRow | undefined, unit: string) {
  if (!product || !unit) return "1";
  if (unit === product.package_unit && product.package_multiplier) return formatQuantityInput(product.package_multiplier);
  if (unit === product.main_unit) return "1";
  return "1";
}

function lotOptionLabel(lot: InventoryLotRow, unit: string) {
  const parts = [lot.lot_number, formatQuantity(lot.quantity, unit)];
  if (lot.expiry_date) parts.push(`SKT ${formatDate(lot.expiry_date)}`);
  if (lot.supplier) parts.push(lot.supplier);
  return parts.join(" / ");
}

function addShipmentDraftItem(
  products: InventoryProductRow[],
  values: Record<string, string>,
  setValues: Dispatch<SetStateAction<Record<string, string>>>,
  setItems: Dispatch<SetStateAction<ShipmentDraftItem[]>>
) {
  const product = products.find((item) => String(item.id) === values.product_id);
  const quantity = Number(values.quantity);
  const multiplier = Number(values.unit_multiplier || 1);
  if (!product || !Number.isFinite(quantity) || quantity <= 0 || !Number.isFinite(multiplier) || multiplier <= 0) {
    void appAlert("Sevkiyat kalemi için ürün, miktar ve çarpan geçerli olmalı.", {
      title: "Geçersiz sevkiyat kalemi"
    });
    return;
  }
  const unit = values.unit || product.main_unit;
  setItems((current) => [
    ...current,
    {
      product_id: product.id,
      product_name: product.product_name,
      quantity,
      unit,
      unit_multiplier: multiplier,
      base_quantity: quantity * multiplier
    }
  ]);
  setValues((current) => ({
    ...current,
    product_id: "",
    quantity: "",
    unit: "adet",
    unit_multiplier: "1"
  }));
}

function addCountDraftItem(
  products: InventoryProductRow[],
  values: Record<string, string>,
  setValues: Dispatch<SetStateAction<Record<string, string>>>,
  setItems: Dispatch<SetStateAction<CountDraftItem[]>>
) {
  const product = products.find((item) => String(item.id) === values.product_id);
  const countedStock = Number(values.counted_stock);
  if (!product || !Number.isFinite(countedStock) || countedStock < 0) {
    void appAlert("Sayım kalemi için ürün ve sayım stoğu geçerli olmalı.", {
      title: "Geçersiz sayım kalemi"
    });
    return;
  }
  setItems((current) => [
    ...current,
    {
      product_id: product.id,
      product_name: product.product_name,
      counted_stock: countedStock,
      reason: values.reason || undefined
    }
  ]);
  setValues((current) => ({
    ...current,
    product_id: "",
    counted_stock: "",
    reason: ""
  }));
}

function startProductEdit(
  product: InventoryProductRow,
  setEditingProductId: Dispatch<SetStateAction<number | null>>,
  setProductValues: Dispatch<SetStateAction<Record<string, string>>>
) {
  setEditingProductId(product.id);
  setProductValues({
    product_name: product.product_name,
    category: product.category ?? "",
    sub_category: product.sub_category ?? "",
    brand: product.brand ?? "",
    description: product.description ?? "",
    main_unit: product.main_unit,
    package_unit: product.package_unit ?? "",
    package_multiplier: formatQuantityInput(product.package_multiplier),
    minimum_stock: formatQuantityInput(product.minimum_stock),
    maximum_stock: formatQuantityInput(product.maximum_stock),
    critical_stock: formatQuantityInput(product.critical_stock),
    safety_stock: formatQuantityInput(product.safety_stock),
    unit_cost: formatQuantityInput(product.unit_cost),
    is_lot_tracked: product.is_lot_tracked ? "true" : "false",
    expiry_tracking: product.expiry_tracking ? "true" : "false"
  });
}

function buildProductPayload(values: Record<string, string>) {
  return typedPayload(
    values,
    [
    "package_multiplier",
    "minimum_stock",
    "maximum_stock",
    "critical_stock",
    "safety_stock",
    "unit_cost"
    ],
    ["is_lot_tracked", "expiry_tracking"]
  );
}

function buildMovementPayload(values: Record<string, string>) {
  return numericPayload(values, ["product_id", "lot_id", "quantity", "unit_multiplier", "branch_id", "vehicle_id", "reference_id"]);
}

function buildLotPayload(values: Record<string, string>) {
  return numericPayload(values, ["product_id", "quantity"]);
}

function buildShipmentPayload(values: Record<string, string>, items: ShipmentDraftItem[]) {
  if (items.length === 0) {
    throw new Error("Sevkiyat için en az bir kalem eklenmeli.");
  }
  return {
    branch_id: values.branch_id ? Number(values.branch_id) : undefined,
    note: values.note || undefined,
    items: items.map((item) => ({
      product_id: item.product_id,
      quantity: item.quantity,
      unit: item.unit,
      unit_multiplier: item.unit_multiplier
    }))
  };
}

function buildCountPayload(values: Record<string, string>, items: CountDraftItem[]) {
  if (items.length === 0) {
    throw new Error("Sayım için en az bir kalem eklenmeli.");
  }
  return {
    count_type: values.count_type,
    count_method: values.count_method,
    category: values.category || undefined,
    note: values.note || undefined,
    items: items.map((item) => ({
      product_id: item.product_id,
      counted_stock: item.counted_stock,
      reason: item.reason
    }))
  };
}

function buildPurchaseRequestPayload(values: Record<string, string>) {
  return numericPayload(values, ["product_id", "requested_quantity"]);
}

async function buildPurchaseRequestStatusUpdate(request: InventoryPurchaseRequestRow, status: string) {
  const payload: Record<string, unknown> = { request_status: status };
  if (status === "approved") {
    const approvedQuantity = await appPrompt("Onay miktarı", {
      title: "Satın alma talebini onayla",
      defaultValue: formatQuantityInput(request.approved_quantity ?? request.requested_quantity),
      confirmLabel: "Onayla"
    });
    if (approvedQuantity === null) return null;
    const parsed = Number(approvedQuantity);
    if (!Number.isFinite(parsed) || parsed < 0) {
      await appAlert("Onay miktarı geçersiz.", {
        title: "Geçersiz miktar"
      });
      return null;
    }
    payload.approved_quantity = parsed;
  }
  return { id: request.id, payload };
}

function buildQueryString(values: Record<string, string | undefined>) {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(values)) {
    const normalized = value?.trim();
    if (normalized) params.set(key, normalized);
  }
  return params.toString();
}

function typedPayload(values: Record<string, string>, numericKeys: string[], booleanKeys: string[]) {
  const payload = numericPayload(values, numericKeys);
  for (const key of booleanKeys) {
    if (values[key] === "true") payload[key] = true;
    if (values[key] === "false") payload[key] = false;
  }
  return payload;
}

function numericPayload(values: Record<string, string>, numericKeys: string[]) {
  const payload: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(values)) {
    if (value === "") continue;
    payload[key] = numericKeys.includes(key) ? Number(value) : value;
  }
  return payload;
}
