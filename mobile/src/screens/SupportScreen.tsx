import * as ImagePicker from "expo-image-picker";
import { useCallback, useEffect, useState } from "react";
import { RefreshControl, ScrollView, StyleSheet, Text, View } from "react-native";
import type { ApiClient } from "../api";
import { Badge, Card, EmptyText, ErrorText, Field, PrimaryButton, Screen, SecondaryButton } from "../components";
import { canUse } from "../permissions";
import { colors } from "../theme";
import type { CurrentUser, SupportTicket, Vehicle } from "../types";

const priorities = ["low", "medium", "high", "critical"] as const;
const maxMediaSize = 25 * 1024 * 1024;

export function SupportScreen({ api, user }: { api: ApiClient; user: CurrentUser }) {
  const [tickets, setTickets] = useState<SupportTicket[]>([]);
  const [vehicles, setVehicles] = useState<Vehicle[]>([]);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [reporterPhone, setReporterPhone] = useState("");
  const [priority, setPriority] = useState<(typeof priorities)[number]>("medium");
  const [faultPlate, setFaultPlate] = useState("");
  const [faultReason, setFaultReason] = useState("");
  const [faultDescription, setFaultDescription] = useState("");
  const [ticketMedia, setTicketMedia] = useState<ImagePicker.ImagePickerAsset | null>(null);
  const [faultMedia, setFaultMedia] = useState<ImagePicker.ImagePickerAsset | null>(null);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [savingFault, setSavingFault] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const canCreateTicket = canUse(user, "support_ticket", "create");
  const canCreateVehicleFault = canUse(user, "vehicle_fault", "create");
  const canUploadMedia = canUse(user, "media_upload", "create");

  const load = useCallback(async () => {
    setError(null);
    setLoading(true);
    try {
      const [nextTickets, nextVehicles] = await Promise.all([
        api.supportTickets(),
        canUse(user, "vehicle_view", "view") ? api.vehicles() : Promise.resolve([])
      ]);
      setTickets(nextTickets);
      setVehicles(nextVehicles);
    } catch (event) {
      setError(event instanceof Error ? event.message : "Destek talepleri alınamadı");
    } finally {
      setLoading(false);
    }
  }, [api, user]);

  useEffect(() => {
    void load();
  }, [load]);

  const createTicket = async () => {
    if (!title.trim() || !description.trim()) {
      setError("Başlık ve açıklama zorunlu");
      return;
    }

    setError(null);
    setSaving(true);
    try {
      const ticket = await api.createSupportTicket({
        title: title.trim(),
        description: description.trim(),
        priority,
        source_channel: "mobile",
        category: "it_support",
        reporter_phone: reporterPhone.trim() || undefined
      });
      if (ticketMedia && canUploadMedia) {
        await api.uploadFile({
          module_name: "support_tickets",
          entity_id: ticket.id,
          file_type: ticketMedia.type === "video" ? "support_video" : "support_screenshot",
          note: "Mobil destek talebi eki",
          uri: ticketMedia.uri,
          name: ticketMedia.fileName ?? `mobile-support-${Date.now()}.${ticketMedia.type === "video" ? "mp4" : "jpg"}`,
          mimeType: ticketMedia.mimeType ?? (ticketMedia.type === "video" ? "video/mp4" : "image/jpeg")
        });
      }
      setTitle("");
      setDescription("");
      setReporterPhone("");
      setPriority("medium");
      setTicketMedia(null);
      await load();
    } catch (event) {
      setError(event instanceof Error ? event.message : "Destek talebi oluşturulamadı");
    } finally {
      setSaving(false);
    }
  };

  const pickMedia = async (target: "ticket" | "fault", source: "camera" | "library") => {
    setError(null);
    const permission =
      source === "camera"
        ? await ImagePicker.requestCameraPermissionsAsync()
        : await ImagePicker.requestMediaLibraryPermissionsAsync();
    if (!permission.granted) {
      setError(source === "camera" ? "Kamera izni gerekli" : "Fotoğraf/video seçimi için izin gerekli");
      return;
    }
    const pickerOptions: ImagePicker.ImagePickerOptions = {
      mediaTypes: ImagePicker.MediaTypeOptions.All,
      quality: 0.7,
      videoMaxDuration: 20
    };
    const result =
      source === "camera"
        ? await ImagePicker.launchCameraAsync(pickerOptions)
        : await ImagePicker.launchImageLibraryAsync(pickerOptions);
    if (result.canceled) return;
    const asset = result.assets[0];
    if (asset.fileSize && asset.fileSize > maxMediaSize) {
      setError("Medya dosyası 25 MB altında olmalı");
      return;
    }
    if (target === "ticket") {
      setTicketMedia(asset);
    } else {
      setFaultMedia(asset);
    }
  };

  const createVehicleFault = async () => {
    if (!faultPlate.trim() || !faultReason.trim()) {
      setError("Plaka ve arıza sebebi zorunlu");
      return;
    }

    setError(null);
    setSavingFault(true);
    try {
      const ticket = await api.createSupportTicket({
        title: `Araç arıza bildirimi: ${faultPlate.trim().toUpperCase()}`,
        description: [
          `Plaka: ${faultPlate.trim().toUpperCase()}`,
          `Arıza sebebi: ${faultReason.trim()}`,
          faultDescription.trim() ? `Açıklama: ${faultDescription.trim()}` : null
        ].filter(Boolean).join("\n"),
        priority: "high",
        source_channel: "mobile",
        category: "vehicle_fault",
        reporter_phone: reporterPhone.trim() || undefined
      });
      if (faultMedia && canUploadMedia) {
        await api.uploadFile({
          module_name: "support_tickets",
          entity_id: ticket.id,
          file_type: faultMedia.type === "video" ? "support_video" : "support_screenshot",
          note: `Mobil arıza bildirimi ${faultPlate.trim().toUpperCase()}`,
          uri: faultMedia.uri,
          name: faultMedia.fileName ?? `mobile-fault-${Date.now()}.${faultMedia.type === "video" ? "mp4" : "jpg"}`,
          mimeType: faultMedia.mimeType ?? (faultMedia.type === "video" ? "video/mp4" : "image/jpeg")
        });
      }
      setFaultPlate("");
      setFaultReason("");
      setFaultDescription("");
      setFaultMedia(null);
      await load();
    } catch (event) {
      setError(event instanceof Error ? event.message : "Arıza bildirimi oluşturulamadı");
    } finally {
      setSavingFault(false);
    }
  };

  return (
    <Screen title="IT Destek" subtitle="Mobil destek talebi ve durum takibi" action={<SecondaryButton label="Yenile" onPress={load} />}>
      <ScrollView refreshControl={<RefreshControl refreshing={loading} onRefresh={load} />}>
        <ErrorText message={error} />

        {canCreateVehicleFault ? (
          <Card>
            <Text style={styles.sectionTitle}>Araç Servis / Arıza</Text>
            <Field label="Araç plakası" value={faultPlate} onChangeText={setFaultPlate} />
            <Field label="Arıza sebebi" value={faultReason} onChangeText={setFaultReason} />
            <Field label="Açıklama" value={faultDescription} onChangeText={setFaultDescription} multiline />
            {canUploadMedia ? (
              <View style={styles.mediaRow}>
                <SecondaryButton label="Kamera" onPress={() => pickMedia("fault", "camera")} />
                <SecondaryButton label="Galeri" onPress={() => pickMedia("fault", "library")} />
                {faultMedia ? <Text style={styles.mediaName}>{faultMedia.fileName ?? faultMedia.uri.split("/").pop()}</Text> : null}
              </View>
            ) : null}
            <PrimaryButton label="Arıza Bildir" onPress={createVehicleFault} loading={savingFault} />
            {vehicles.length > 0 ? (
              <View style={styles.vehicleHints}>
                {vehicles.slice(0, 5).map((vehicle) => (
                  <SecondaryButton key={vehicle.id} label={vehicle.plate} onPress={() => setFaultPlate(vehicle.plate)} />
                ))}
              </View>
            ) : null}
          </Card>
        ) : null}

        {canCreateTicket ? (
          <Card>
            <Text style={styles.sectionTitle}>Yeni Talep</Text>
            <Field label="Başlık" value={title} onChangeText={setTitle} />
            <Field label="Açıklama" value={description} onChangeText={setDescription} multiline />
            <Field label="Telefon" value={reporterPhone} onChangeText={setReporterPhone} keyboardType="phone-pad" />
            {canUploadMedia ? (
              <View style={styles.mediaRow}>
                <SecondaryButton label="Kamera" onPress={() => pickMedia("ticket", "camera")} />
                <SecondaryButton label="Galeri" onPress={() => pickMedia("ticket", "library")} />
                {ticketMedia ? <Text style={styles.mediaName}>{ticketMedia.fileName ?? ticketMedia.uri.split("/").pop()}</Text> : null}
              </View>
            ) : null}
            <View style={styles.priorityRow}>
              {priorities.map((item) => (
                <SecondaryButton key={item} label={priority === item ? `${priorityLabel(item)} ✓` : priorityLabel(item)} onPress={() => setPriority(item)} />
              ))}
            </View>
            <PrimaryButton label="Talep Oluştur" onPress={createTicket} loading={saving} />
          </Card>
        ) : null}

        <Card>
          <Text style={styles.sectionTitle}>Talepler</Text>
          {tickets.length === 0 ? <EmptyText message="Destek talebi yok." /> : null}
          {tickets.slice(0, 30).map((ticket) => (
            <View key={ticket.id} style={styles.ticketRow}>
              <View style={styles.ticketText}>
                <Text style={styles.ticketTitle}>{ticket.ticket_no} · {ticket.title}</Text>
                <Text style={styles.ticketDetail}>{ticket.description}</Text>
                <Text style={styles.ticketMeta}>
                  {statusLabel(ticket.ticket_status)} / {priorityLabel(ticket.priority)} / SLA: {slaLabel(ticket.sla_status)}
                </Text>
                <Text style={styles.ticketMeta}>Çözüm hedefi: {formatDate(ticket.sla_resolution_due_at)}</Text>
                {ticket.resolution_note ? <Text style={styles.resolution}>Çözüm: {ticket.resolution_note}</Text> : null}
              </View>
              <Badge value={statusLabel(ticket.ticket_status)} />
            </View>
          ))}
        </Card>
      </ScrollView>
    </Screen>
  );
}

function priorityLabel(value: string) {
  const labels: Record<string, string> = {
    low: "Düşük",
    medium: "Orta",
    high: "Yüksek",
    critical: "Kritik"
  };
  return labels[value] ?? value;
}

function statusLabel(value: string) {
  const labels: Record<string, string> = {
    open: "Açık",
    in_progress: "İşlemde",
    waiting_user: "Kullanıcı Bekleniyor",
    resolved: "Çözüldü",
    closed: "Kapandı",
    cancelled: "İptal"
  };
  return labels[value] ?? value;
}

function slaLabel(value: string) {
  const labels: Record<string, string> = {
    on_track: "Normal",
    due_soon: "Yaklaşıyor",
    response_overdue: "Cevap Gecikti",
    resolution_overdue: "Çözüm Gecikti",
    closed: "Kapalı"
  };
  return labels[value] ?? value;
}

function formatDate(value: string | null) {
  if (!value) return "-";
  return new Date(value).toLocaleString("tr-TR");
}

const styles = StyleSheet.create({
  sectionTitle: {
    color: colors.text,
    fontWeight: "800",
    marginBottom: 10
  },
  priorityRow: {
    flexDirection: "row",
    flexWrap: "wrap",
    gap: 8,
    marginBottom: 12
  },
  mediaRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: 10,
    marginBottom: 12
  },
  mediaName: {
    flex: 1,
    color: colors.textMuted,
    fontSize: 12
  },
  vehicleHints: {
    flexDirection: "row",
    flexWrap: "wrap",
    gap: 8,
    marginTop: 12
  },
  ticketRow: {
    flexDirection: "row",
    gap: 10,
    justifyContent: "space-between",
    alignItems: "flex-start",
    borderTopWidth: 1,
    borderTopColor: colors.border,
    paddingVertical: 12
  },
  ticketText: {
    flex: 1
  },
  ticketTitle: {
    color: colors.text,
    fontWeight: "900"
  },
  ticketDetail: {
    color: colors.textMuted,
    marginTop: 5,
    fontSize: 13
  },
  ticketMeta: {
    color: colors.textMuted,
    marginTop: 4,
    fontSize: 12
  },
  resolution: {
    color: colors.text,
    marginTop: 6,
    fontSize: 12,
    fontWeight: "700"
  }
});
