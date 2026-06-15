import * as ImagePicker from "expo-image-picker";
import { useCallback, useEffect, useState } from "react";
import { RefreshControl, ScrollView, StyleSheet, Text, View } from "react-native";
import type { ApiClient } from "../api";
import { Badge, Card, EmptyText, ErrorText, Field, PrimaryButton, Screen, SecondaryButton } from "../components";
import { canUse } from "../permissions";
import { colors } from "../theme";
import type { CurrentUser, Vehicle } from "../types";

const maxKmPhotoSize = 10 * 1024 * 1024;

export function VehiclesScreen({ api, user }: { api: ApiClient; user: CurrentUser }) {
  const [vehicles, setVehicles] = useState<Vehicle[]>([]);
  const [selectedVehicleId, setSelectedVehicleId] = useState<number | null>(null);
  const [kmValue, setKmValue] = useState("");
  const [kmPhoto, setKmPhoto] = useState<ImagePicker.ImagePickerAsset | null>(null);
  const [loading, setLoading] = useState(false);
  const [savingKm, setSavingKm] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    setLoading(true);
    try {
      setVehicles(await api.vehicles());
    } catch (event) {
      setError(event instanceof Error ? event.message : "Araçlar alınamadı");
    } finally {
      setLoading(false);
    }
  }, [api]);

  useEffect(() => {
    void load();
  }, [load]);

  const selectedVehicle = vehicles.find((vehicle) => vehicle.id === selectedVehicleId);
  const canCreateKm = canUse(user, "km_log", "create");

  const pickKmPhoto = async (source: "camera" | "library") => {
    setError(null);
    const permission =
      source === "camera"
        ? await ImagePicker.requestCameraPermissionsAsync()
        : await ImagePicker.requestMediaLibraryPermissionsAsync();
    if (!permission.granted) {
      setError(source === "camera" ? "Kamera izni gerekli" : "KM fotoğrafı seçimi için izin gerekli");
      return;
    }
    const pickerOptions: ImagePicker.ImagePickerOptions = {
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      quality: 0.8
    };
    const result =
      source === "camera"
        ? await ImagePicker.launchCameraAsync(pickerOptions)
        : await ImagePicker.launchImageLibraryAsync(pickerOptions);
    if (result.canceled) return;
    const asset = result.assets[0];
    if (asset.fileSize && asset.fileSize > maxKmPhotoSize) {
      setError("KM fotoğrafı 10 MB altında olmalı");
      return;
    }
    setKmPhoto(asset);
  };

  const createKmLog = async () => {
    if (!selectedVehicleId) {
      setError("Önce araç seçilmeli");
      return;
    }
    if (!kmValue) {
      setError("KM girilmeli");
      return;
    }

    setError(null);
    setSavingKm(true);
    try {
      let imagePath: string | undefined;
      let fileId: number | undefined;
      if (kmPhoto) {
        const uploaded = await api.uploadFile({
          module_name: "vehicles",
          entity_id: selectedVehicleId,
          file_type: "km_photo",
          note: `Mobil KM fotoğrafı ${selectedVehicle?.plate ?? ""}`.trim(),
          uri: kmPhoto.uri,
          name: kmPhoto.fileName ?? `km-photo-${Date.now()}.jpg`,
          mimeType: kmPhoto.mimeType ?? "image/jpeg"
        });
        imagePath = uploaded.storage_path;
        fileId = uploaded.id;
      }
      await api.createKmLog({
        vehicle_id: selectedVehicleId,
        km: Number(kmValue),
        entry_type: kmPhoto ? "ocr" : "manual",
        image_path: imagePath,
        ocr_result: kmPhoto
          ? {
              status: "pending",
              source: "mobile_km_photo",
              file_id: fileId,
              submitted_km: Number(kmValue)
            }
          : undefined,
        device_info: kmPhoto ? "mobile:km-photo" : "mobile"
      });
      setKmValue("");
      setKmPhoto(null);
      await load();
    } catch (event) {
      setError(event instanceof Error ? event.message : "KM kaydı oluşturulamadı");
    } finally {
      setSavingKm(false);
    }
  };

  return (
    <Screen title="Araçlar" subtitle="Mobil hızlı araç listesi" action={<SecondaryButton label="Yenile" onPress={load} />}>
      <ScrollView refreshControl={<RefreshControl refreshing={loading} onRefresh={load} />}>
        <ErrorText message={error} />
        {canCreateKm ? (
          <Card>
            <Text style={styles.sectionTitle}>KM Bildirimi</Text>
            <Text style={styles.detail}>Seçili araç: {selectedVehicle?.plate ?? "Yok"}</Text>
            <Field label="Güncel KM" value={kmValue} onChangeText={setKmValue} keyboardType="numeric" />
            <View style={styles.photoRow}>
              <SecondaryButton label="Kamera" onPress={() => pickKmPhoto("camera")} />
              <SecondaryButton label="Galeri" onPress={() => pickKmPhoto("library")} />
              {kmPhoto ? <Text style={styles.photoName}>{kmPhoto.fileName ?? kmPhoto.uri.split("/").pop()}</Text> : null}
            </View>
            <PrimaryButton label="KM Kaydet" onPress={createKmLog} loading={savingKm} />
          </Card>
        ) : null}
        <Card>
          {vehicles.length === 0 ? <EmptyText message="Araç kaydı yok." /> : null}
          {vehicles.slice(0, 30).map((vehicle) => (
            <View key={vehicle.id} style={styles.row}>
              <View style={styles.rowText}>
                <Text style={styles.plate}>{vehicle.plate}</Text>
                <Text style={styles.detail}>{vehicle.brand} {vehicle.model}{vehicle.model_year ? ` · ${vehicle.model_year}` : ""}</Text>
                <Text style={styles.detail}>{vehicle.department_name ?? vehicle.user_name ?? "Atama bilgisi yok"}</Text>
              </View>
              <View style={styles.actions}>
                <Badge value={vehicle.status} />
                <SecondaryButton label={selectedVehicleId === vehicle.id ? "Seçili" : "Seç"} onPress={() => setSelectedVehicleId(vehicle.id)} />
              </View>
            </View>
          ))}
        </Card>
      </ScrollView>
    </Screen>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    gap: 10,
    justifyContent: "space-between",
    alignItems: "center",
    borderTopWidth: 1,
    borderTopColor: colors.border,
    paddingVertical: 10
  },
  rowText: {
    flex: 1
  },
  sectionTitle: {
    color: colors.text,
    fontWeight: "800",
    marginBottom: 10
  },
  actions: {
    alignItems: "flex-end",
    gap: 8
  },
  plate: {
    color: colors.text,
    fontWeight: "900",
    fontSize: 16
  },
  detail: {
    color: colors.textMuted,
    marginTop: 3,
    fontSize: 12
  },
  photoRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: 10,
    marginBottom: 12
  },
  photoName: {
    flex: 1,
    color: colors.textMuted,
    fontSize: 12
  }
});
