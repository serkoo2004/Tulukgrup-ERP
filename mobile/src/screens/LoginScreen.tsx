import { useState } from "react";
import { KeyboardAvoidingView, Platform, StyleSheet, Text, View } from "react-native";
import type { ApiClient } from "../api";
import { ErrorText, Field, PrimaryButton } from "../components";
import { colors } from "../theme";
import type { AuthSession } from "../types";

export function LoginScreen({
  api,
  apiBaseUrl,
  onApiBaseUrlChange,
  onLogin
}: {
  api: ApiClient;
  apiBaseUrl: string;
  onApiBaseUrlChange: (value: string) => void;
  onLogin: (session: AuthSession) => void;
}) {
  const [email, setEmail] = useState("admin@tuluklar.local");
  const [password, setPassword] = useState("Admin12345!");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    setError(null);
    setLoading(true);
    try {
      const session = await api.login(email.trim(), password);
      onLogin(session);
    } catch (event) {
      setError(event instanceof Error ? event.message : "Giriş yapılamadı");
    } finally {
      setLoading(false);
    }
  };

  return (
    <KeyboardAvoidingView behavior={Platform.OS === "ios" ? "padding" : undefined} style={styles.root}>
      <View style={styles.brand}>
        <Text style={styles.brandTitle}>Tuluklar ERP</Text>
        <Text style={styles.brandSubtitle}>Mobil operasyon paneli</Text>
      </View>
      <View style={styles.card}>
        <Field label="E-posta" value={email} onChangeText={setEmail} keyboardType="email-address" />
        <Field label="Şifre" value={password} onChangeText={setPassword} secureTextEntry />
        <Field label="API adresi" value={apiBaseUrl} onChangeText={onApiBaseUrlChange} />
        <ErrorText message={error} />
        <PrimaryButton label="Giriş Yap" onPress={submit} loading={loading} />
      </View>
      <Text style={styles.note}>Telefon testinde Tailscale veya Wi-Fi backend adresini kullan.</Text>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  root: {
    flex: 1,
    justifyContent: "center",
    padding: 18
  },
  brand: {
    marginBottom: 18
  },
  brandTitle: {
    fontSize: 28,
    fontWeight: "900",
    color: colors.text
  },
  brandSubtitle: {
    color: colors.textMuted,
    marginTop: 5
  },
  card: {
    borderRadius: 8,
    borderWidth: 1,
    borderColor: colors.border,
    backgroundColor: colors.surface,
    padding: 16
  },
  note: {
    marginTop: 14,
    color: colors.textMuted,
    fontSize: 12
  }
});
