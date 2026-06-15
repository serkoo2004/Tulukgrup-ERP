import { useCallback, useEffect, useState } from "react";
import { RefreshControl, ScrollView, StyleSheet, Text, View } from "react-native";
import type { ApiClient } from "../api";
import { Badge, Card, EmptyText, ErrorText, Field, Metric, PrimaryButton, Screen, SecondaryButton } from "../components";
import { canUse } from "../permissions";
import { colors } from "../theme";
import type { CurrentUser, InventoryAlert, InventoryDashboard, InventoryProduct, InventoryPurchaseRequest } from "../types";

export function InventoryScreen({ api, user }: { api: ApiClient; user: CurrentUser }) {
  const [dashboard, setDashboard] = useState<InventoryDashboard | null>(null);
  const [products, setProducts] = useState<InventoryProduct[]>([]);
  const [alerts, setAlerts] = useState<InventoryAlert[]>([]);
  const [purchaseRequests, setPurchaseRequests] = useState<InventoryPurchaseRequest[]>([]);
  const [productName, setProductName] = useState("");
  const [category, setCategory] = useState("");
  const [minimumStock, setMinimumStock] = useState("");
  const [selectedProductId, setSelectedProductId] = useState<number | null>(null);
  const [movementType, setMovementType] = useState<"warehouse_in" | "warehouse_out">("warehouse_in");
  const [movementQuantity, setMovementQuantity] = useState("");
  const [orderQuantity, setOrderQuantity] = useState("");
  const [orderReason, setOrderReason] = useState("");
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [savingMovement, setSavingMovement] = useState(false);
  const [savingOrder, setSavingOrder] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    setLoading(true);
    try {
      const [nextDashboard, nextProducts, nextAlerts, nextPurchaseRequests] = await Promise.all([
        api.inventoryDashboard(),
        api.inventoryProducts(),
        api.inventoryAlerts(),
        api.inventoryPurchaseRequests()
      ]);
      setDashboard(nextDashboard);
      setProducts(nextProducts);
      setAlerts(nextAlerts);
      setPurchaseRequests(nextPurchaseRequests);
    } catch (event) {
      setError(event instanceof Error ? event.message : "Stok verisi alınamadı");
    } finally {
      setLoading(false);
    }
  }, [api]);

  useEffect(() => {
    void load();
  }, [load]);

  const createProduct = async () => {
    if (!productName.trim()) {
      setError("Ürün adı zorunlu");
      return;
    }
    if (minimumStock && Number(minimumStock) < 0) {
      setError("Minimum stok negatif olamaz");
      return;
    }

    setError(null);
    setSaving(true);
    try {
      await api.createInventoryProduct({
        product_name: productName.trim(),
        category: category.trim() || undefined,
        main_unit: "adet",
        package_unit: "koli",
        package_multiplier: 1,
        minimum_stock: minimumStock ? Number(minimumStock) : 0
      });
      setProductName("");
      setCategory("");
      setMinimumStock("");
      await load();
    } catch (event) {
      setError(event instanceof Error ? event.message : "Ürün eklenemedi");
    } finally {
      setSaving(false);
    }
  };

  const createMovement = async () => {
    if (!selectedProductId) {
      setError("Önce ürün seçilmeli");
      return;
    }
    if (!movementQuantity) {
      setError("Miktar girilmeli");
      return;
    }
    if (Number(movementQuantity) <= 0) {
      setError("Miktar sıfırdan büyük olmalı");
      return;
    }

    setError(null);
    setSavingMovement(true);
    try {
      await api.createInventoryMovement({
        product_id: selectedProductId,
        movement_type: movementType,
        quantity: Number(movementQuantity),
        unit: "adet",
        unit_multiplier: 1,
        description: "Mobil stok hareketi"
      });
      setMovementQuantity("");
      await load();
    } catch (event) {
      setError(event instanceof Error ? event.message : "Stok hareketi kaydedilemedi");
    } finally {
      setSavingMovement(false);
    }
  };

  const createPurchaseRequest = async () => {
    if (!selectedProductId) {
      setError("Önce ürün seçilmeli");
      return;
    }
    if (!orderQuantity) {
      setError("Sipariş miktarı girilmeli");
      return;
    }
    if (Number(orderQuantity) <= 0) {
      setError("Sipariş miktarı sıfırdan büyük olmalı");
      return;
    }

    setError(null);
    setSavingOrder(true);
    try {
      await api.createInventoryPurchaseRequest({
        product_id: selectedProductId,
        requested_quantity: Number(orderQuantity),
        reason: orderReason.trim() || "Mobil ürün siparişi",
        note: "Mobil uygulama üzerinden oluşturuldu"
      });
      setOrderQuantity("");
      setOrderReason("");
      await load();
    } catch (event) {
      setError(event instanceof Error ? event.message : "Sipariş talebi oluşturulamadı");
    } finally {
      setSavingOrder(false);
    }
  };

  const selectedProduct = products.find((product) => product.id === selectedProductId);
  const canCreateStock = canUse(user, "inventory_view", "create");
  const canOrder = canUse(user, "inventory_order", "create");

  return (
    <Screen title="Merkez Stok" subtitle="Ürünler, kritik stoklar ve hızlı ürün girişi" action={<SecondaryButton label="Yenile" onPress={load} />}>
      <ScrollView refreshControl={<RefreshControl refreshing={loading} onRefresh={load} />}>
        <ErrorText message={error} />
        {dashboard ? (
          <View style={styles.metrics}>
            <Metric label="Ürün" value={dashboard.total_products} />
            <Metric label="Kritik" value={dashboard.critical_stock_count} tone="warning" />
            <Metric label="Tükenen" value={dashboard.out_of_stock_count} tone="danger" />
            <Metric label="SKT 30 gün" value={dashboard.expiring_lots_30_days} />
          </View>
        ) : null}

        {canCreateStock ? (
          <Card>
            <Text style={styles.sectionTitle}>Ürün Kartı</Text>
            <Field label="Ürün adı" value={productName} onChangeText={setProductName} />
            <Field label="Kategori" value={category} onChangeText={setCategory} />
            <Field label="Minimum stok" value={minimumStock} onChangeText={setMinimumStock} keyboardType="decimal-pad" />
            <PrimaryButton label="Ürün Ekle" onPress={createProduct} loading={saving} />
          </Card>
        ) : null}

        {canOrder ? (
          <Card>
            <Text style={styles.sectionTitle}>Ürün Siparişi</Text>
            <Text style={styles.rowSubtitle}>Seçili ürün: {selectedProduct?.product_name ?? "Yok"}</Text>
            <Field label="Sipariş miktarı" value={orderQuantity} onChangeText={setOrderQuantity} keyboardType="decimal-pad" />
            <Field label="Sipariş nedeni" value={orderReason} onChangeText={setOrderReason} />
            <PrimaryButton label="Sipariş Talebi Oluştur" onPress={createPurchaseRequest} loading={savingOrder} />
          </Card>
        ) : null}

        {canCreateStock ? (
          <Card>
            <Text style={styles.sectionTitle}>Hızlı Stok Hareketi</Text>
            <Text style={styles.rowSubtitle}>Seçili ürün: {selectedProduct?.product_name ?? "Yok"}</Text>
            <View style={styles.modeRow}>
              <SecondaryButton label="Giriş" onPress={() => setMovementType("warehouse_in")} />
              <SecondaryButton label="Çıkış" onPress={() => setMovementType("warehouse_out")} />
              <Badge value={movementType === "warehouse_in" ? "Depo Girişi" : "Depo Çıkışı"} />
            </View>
            <Field label="Miktar" value={movementQuantity} onChangeText={setMovementQuantity} keyboardType="decimal-pad" />
            <PrimaryButton label="Hareket Kaydet" onPress={createMovement} loading={savingMovement} />
          </Card>
        ) : null}

        <Card>
          <Text style={styles.sectionTitle}>Kritik Stok</Text>
          {alerts.length === 0 ? <EmptyText message="Kritik stok kaydı yok." /> : null}
          {alerts.slice(0, 8).map((alert) => (
            <View key={alert.product_id} style={styles.row}>
              <View style={styles.rowText}>
                <Text style={styles.rowTitle}>{alert.product_name}</Text>
                <Text style={styles.rowSubtitle}>Stok: {alert.current_stock} / Min: {alert.minimum_stock}</Text>
              </View>
              <Badge value={alert.alert_level} />
            </View>
          ))}
        </Card>

        <Card>
          <Text style={styles.sectionTitle}>Sipariş Talepleri</Text>
          {purchaseRequests.length === 0 ? <EmptyText message="Sipariş talebi yok." /> : null}
          {purchaseRequests.slice(0, 10).map((request) => (
            <View key={request.id} style={styles.row}>
              <View style={styles.rowText}>
                <Text style={styles.rowTitle}>{request.product_name}</Text>
                <Text style={styles.rowSubtitle}>Talep: {request.requested_quantity} · {request.reason ?? "Neden yok"}</Text>
              </View>
              <Badge value={request.request_status} />
            </View>
          ))}
        </Card>

        <Card>
          <Text style={styles.sectionTitle}>Son Ürünler</Text>
          {products.length === 0 ? <EmptyText message="Ürün kartı yok." /> : null}
          {products.slice(0, 12).map((product) => (
            <View key={product.id} style={styles.row}>
              <View style={styles.rowText}>
                <Text style={styles.rowTitle}>{product.product_name}</Text>
                <Text style={styles.rowSubtitle}>{product.category ?? "Kategori yok"} · {product.current_stock} {product.main_unit}</Text>
              </View>
              <SecondaryButton label={selectedProductId === product.id ? "Seçili" : "Seç"} onPress={() => setSelectedProductId(product.id)} />
            </View>
          ))}
        </Card>
      </ScrollView>
    </Screen>
  );
}

const styles = StyleSheet.create({
  metrics: {
    flexDirection: "row",
    flexWrap: "wrap",
    justifyContent: "space-between"
  },
  sectionTitle: {
    color: colors.text,
    fontWeight: "800",
    marginBottom: 10
  },
  row: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 10,
    borderTopWidth: 1,
    borderTopColor: colors.border,
    paddingVertical: 10
  },
  rowText: {
    flex: 1
  },
  rowTitle: {
    color: colors.text,
    fontWeight: "800"
  },
  rowSubtitle: {
    color: colors.textMuted,
    marginTop: 3,
    fontSize: 12
  },
  modeRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: 8,
    marginTop: 10,
    marginBottom: 10
  }
});
