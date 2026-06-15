import { StyleSheet, Text } from "react-native";
import { Card, PrimaryButton, Screen } from "../components";
import { colors } from "../theme";
import type { AuthSession } from "../types";

export function AccountScreen({ session, onLogout }: { session: AuthSession; onLogout: () => void }) {
  return (
    <Screen title="Hesap" subtitle="Mobil oturum bilgisi">
      <Card>
        <Text style={styles.label}>Token türü</Text>
        <Text style={styles.value}>{session.token_type}</Text>
        <Text style={styles.label}>Oturum</Text>
        <Text style={styles.value}>Aktif</Text>
        <PrimaryButton label="Çıkış Yap" onPress={onLogout} />
      </Card>
    </Screen>
  );
}

const styles = StyleSheet.create({
  label: {
    color: colors.textMuted,
    fontSize: 12,
    fontWeight: "700",
    marginBottom: 4
  },
  value: {
    color: colors.text,
    fontSize: 16,
    fontWeight: "800",
    marginBottom: 14
  }
});
