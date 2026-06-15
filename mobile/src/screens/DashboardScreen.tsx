import { useCallback, useEffect, useState } from "react";
import { RefreshControl, ScrollView, StyleSheet, Text, View } from "react-native";
import type { ApiClient } from "../api";
import { Badge, Card, EmptyText, ErrorText, Metric, Screen, SecondaryButton } from "../components";
import { colors } from "../theme";
import type { DashboardResponse } from "../types";

export function DashboardScreen({ api }: { api: ApiClient }) {
  const [data, setData] = useState<DashboardResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    setLoading(true);
    try {
      setData(await api.dashboard());
    } catch (event) {
      setError(event instanceof Error ? event.message : "Dashboard alınamadı");
    } finally {
      setLoading(false);
    }
  }, [api]);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <Screen title="Dashboard" subtitle={data ? `Son güncelleme: ${new Date(data.generated_at).toLocaleString("tr-TR")}` : "Merkez durum özeti"} action={<SecondaryButton label="Yenile" onPress={load} />}>
      <ScrollView refreshControl={<RefreshControl refreshing={loading} onRefresh={load} />}>
        <ErrorText message={error} />
        {data ? (
          <>
            <View style={styles.metrics}>
              <Metric label="Araç" value={data.vehicles.total_count} />
              <Metric label="Aktif araç" value={data.vehicles.active_count} />
              <Metric label="Açık görev" value={data.tasks.open_count} tone={data.tasks.overdue_count > 0 ? "danger" : "default"} />
              <Metric label="Kritik stok" value={data.inventory.critical_stock_count + data.inventory.out_of_stock_count} tone={data.inventory.out_of_stock_count > 0 ? "danger" : "warning"} />
              <Metric label="IT destek" value={data.support.open_count + data.support.in_progress_count} tone={data.support.resolution_overdue_count > 0 ? "danger" : data.support.due_soon_count > 0 ? "warning" : "default"} />
              <Metric label="WhatsApp" value={data.support.whatsapp_count} />
            </View>
            <Card>
              <Text style={styles.sectionTitle}>Uyarılar</Text>
              {data.warnings.length === 0 ? <EmptyText message="Aktif kritik uyarı yok." /> : null}
              {data.warnings.slice(0, 8).map((warning) => (
                <View key={warning.warning_type} style={styles.warningRow}>
                  <View style={styles.warningText}>
                    <Text style={styles.warningTitle}>{warning.title}</Text>
                    <Badge value={warning.severity} />
                  </View>
                  <Text style={styles.warningCount}>{warning.count}</Text>
                </View>
              ))}
            </Card>
          </>
        ) : null}
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
    marginBottom: 8
  },
  warningRow: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    paddingVertical: 10,
    borderTopWidth: 1,
    borderTopColor: colors.border
  },
  warningText: {
    flex: 1,
    gap: 5
  },
  warningTitle: {
    color: colors.text,
    fontWeight: "700"
  },
  warningCount: {
    color: colors.text,
    fontWeight: "900",
    fontSize: 18
  }
});
