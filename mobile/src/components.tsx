import type { ReactNode } from "react";
import { ActivityIndicator, StyleSheet, Text, TextInput, TouchableOpacity, View } from "react-native";
import { colors } from "./theme";

export function Screen({ title, subtitle, children, action }: { title: string; subtitle?: string; children: ReactNode; action?: ReactNode }) {
  return (
    <View style={styles.screen}>
      <View style={styles.header}>
        <View style={styles.headerText}>
          <Text style={styles.title}>{title}</Text>
          {subtitle ? <Text style={styles.subtitle}>{subtitle}</Text> : null}
        </View>
        {action}
      </View>
      {children}
    </View>
  );
}

export function Card({ children }: { children: ReactNode }) {
  return <View style={styles.card}>{children}</View>;
}

export function Metric({ label, value, tone = "default" }: { label: string; value: string | number; tone?: "default" | "warning" | "danger" }) {
  const toneStyle = tone === "danger" ? styles.metricDanger : tone === "warning" ? styles.metricWarning : styles.metricDefault;
  return (
    <View style={[styles.metric, toneStyle]}>
      <Text style={styles.metricValue}>{value}</Text>
      <Text style={styles.metricLabel}>{label}</Text>
    </View>
  );
}

export function PrimaryButton({ label, onPress, loading }: { label: string; onPress: () => void; loading?: boolean }) {
  return (
    <TouchableOpacity style={styles.primaryButton} onPress={onPress} disabled={loading}>
      {loading ? <ActivityIndicator color={colors.primaryText} /> : <Text style={styles.primaryButtonText}>{label}</Text>}
    </TouchableOpacity>
  );
}

export function SecondaryButton({ label, onPress }: { label: string; onPress: () => void }) {
  return (
    <TouchableOpacity style={styles.secondaryButton} onPress={onPress}>
      <Text style={styles.secondaryButtonText}>{label}</Text>
    </TouchableOpacity>
  );
}

export function Field({ label, value, onChangeText, secureTextEntry, keyboardType = "default", multiline }: {
  label: string;
  value: string;
  onChangeText: (value: string) => void;
  secureTextEntry?: boolean;
  keyboardType?: "default" | "email-address" | "numeric" | "decimal-pad" | "phone-pad";
  multiline?: boolean;
}) {
  return (
    <View style={styles.field}>
      <Text style={styles.fieldLabel}>{label}</Text>
      <TextInput
        style={[styles.input, multiline && styles.multilineInput]}
        value={value}
        onChangeText={onChangeText}
        secureTextEntry={secureTextEntry}
        keyboardType={keyboardType}
        multiline={multiline}
        textAlignVertical={multiline ? "top" : "center"}
        autoCapitalize="none"
        placeholderTextColor={colors.textMuted}
      />
    </View>
  );
}

export function ErrorText({ message }: { message?: string | null }) {
  if (!message) return null;
  return <Text style={styles.error}>{message}</Text>;
}

export function EmptyText({ message }: { message: string }) {
  return <Text style={styles.empty}>{message}</Text>;
}

export function Badge({ value }: { value: string }) {
  return (
    <View style={styles.badge}>
      <Text style={styles.badgeText}>{value}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  screen: {
    flex: 1,
    padding: 16
  },
  header: {
    flexDirection: "row",
    alignItems: "flex-start",
    justifyContent: "space-between",
    gap: 12,
    marginBottom: 14
  },
  headerText: {
    flex: 1
  },
  title: {
    fontSize: 22,
    fontWeight: "800",
    color: colors.text
  },
  subtitle: {
    marginTop: 4,
    fontSize: 13,
    color: colors.textMuted
  },
  card: {
    padding: 14,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: colors.border,
    backgroundColor: colors.surface,
    marginBottom: 12
  },
  metric: {
    flex: 1,
    minWidth: "47%",
    padding: 12,
    borderRadius: 8,
    borderWidth: 1,
    marginBottom: 10
  },
  metricDefault: {
    backgroundColor: colors.surface,
    borderColor: colors.border
  },
  metricWarning: {
    backgroundColor: colors.warningSoft,
    borderColor: "#fde68a"
  },
  metricDanger: {
    backgroundColor: colors.dangerSoft,
    borderColor: "#fecaca"
  },
  metricValue: {
    color: colors.text,
    fontSize: 20,
    fontWeight: "800"
  },
  metricLabel: {
    color: colors.textMuted,
    marginTop: 4,
    fontSize: 12
  },
  primaryButton: {
    minHeight: 46,
    borderRadius: 8,
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: colors.primary,
    paddingHorizontal: 14
  },
  primaryButtonText: {
    color: colors.primaryText,
    fontWeight: "800"
  },
  secondaryButton: {
    minHeight: 40,
    borderRadius: 8,
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: colors.muted,
    paddingHorizontal: 12
  },
  secondaryButtonText: {
    color: colors.text,
    fontWeight: "700"
  },
  field: {
    marginBottom: 10
  },
  fieldLabel: {
    color: colors.textMuted,
    fontSize: 12,
    marginBottom: 5,
    fontWeight: "700"
  },
  input: {
    minHeight: 44,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: colors.border,
    backgroundColor: colors.surface,
    paddingHorizontal: 12,
    color: colors.text,
    fontSize: 15
  },
  multilineInput: {
    minHeight: 92,
    paddingTop: 10
  },
  error: {
    color: colors.danger,
    marginBottom: 10,
    fontWeight: "700"
  },
  empty: {
    color: colors.textMuted,
    textAlign: "center",
    paddingVertical: 18
  },
  badge: {
    alignSelf: "flex-start",
    backgroundColor: colors.muted,
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 6
  },
  badgeText: {
    color: colors.textMuted,
    fontSize: 12,
    fontWeight: "700"
  }
});
