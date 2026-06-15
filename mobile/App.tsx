import { StatusBar } from "expo-status-bar";
import { useEffect, useMemo, useState } from "react";
import { SafeAreaView, StyleSheet, Text, TouchableOpacity, View } from "react-native";
import { ApiClient, defaultApiBaseUrl } from "./src/api";
import { ErrorText } from "./src/components";
import { visibleTabs } from "./src/permissions";
import { AccountScreen } from "./src/screens/AccountScreen";
import { DashboardScreen } from "./src/screens/DashboardScreen";
import { InventoryScreen } from "./src/screens/InventoryScreen";
import { LoginScreen } from "./src/screens/LoginScreen";
import { SupportScreen } from "./src/screens/SupportScreen";
import { VehiclesScreen } from "./src/screens/VehiclesScreen";
import { colors } from "./src/theme";
import type { AuthSession, CurrentUser, MobileTab } from "./src/types";

const tabs: Array<{ key: MobileTab; label: string }> = [
  { key: "dashboard", label: "Panel" },
  { key: "inventory", label: "Stok" },
  { key: "support", label: "Destek" },
  { key: "vehicles", label: "Araçlar" },
  { key: "account", label: "Hesap" }
];

export default function App() {
  const [session, setSession] = useState<AuthSession | null>(null);
  const [currentUser, setCurrentUser] = useState<CurrentUser | null>(null);
  const [activeTab, setActiveTab] = useState<MobileTab>("dashboard");
  const [profileError, setProfileError] = useState<string | null>(null);
  const [apiBaseUrl, setApiBaseUrl] = useState(defaultApiBaseUrl());
  const api = useMemo(() => new ApiClient(() => session?.access_token ?? null, () => apiBaseUrl), [session, apiBaseUrl]);
  const availableTabs = useMemo(() => visibleTabs(currentUser, tabs), [currentUser]);

  useEffect(() => {
    if (!session) {
      setCurrentUser(null);
      return;
    }
    let mounted = true;
    setProfileError(null);
    api
      .me()
      .then((profile) => {
        if (mounted) setCurrentUser(profile);
      })
      .catch((event) => {
        if (mounted) setProfileError(event instanceof Error ? event.message : "Profil alınamadı");
      });
    return () => {
      mounted = false;
    };
  }, [api, session]);

  useEffect(() => {
    if (availableTabs.length > 0 && !availableTabs.some((tab) => tab.key === activeTab)) {
      setActiveTab(availableTabs[0].key);
    }
  }, [activeTab, availableTabs]);

  if (!session) {
    return (
      <SafeAreaView style={styles.root}>
        <StatusBar style="dark" />
        <LoginScreen api={api} apiBaseUrl={apiBaseUrl} onApiBaseUrlChange={setApiBaseUrl} onLogin={setSession} />
      </SafeAreaView>
    );
  }

  if (!currentUser) {
    return (
      <SafeAreaView style={styles.root}>
        <StatusBar style="dark" />
        <View style={styles.loadingProfile}>
          <Text style={styles.loadingTitle}>Profil yükleniyor</Text>
          <ErrorText message={profileError} />
          {profileError ? (
            <TouchableOpacity style={styles.logoutButton} onPress={() => setSession(null)}>
              <Text style={styles.logoutText}>Girişe Dön</Text>
            </TouchableOpacity>
          ) : null}
        </View>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.root}>
      <StatusBar style="dark" />
      <View style={styles.content}>
        {activeTab === "dashboard" && <DashboardScreen api={api} />}
        {activeTab === "inventory" && <InventoryScreen api={api} user={currentUser} />}
        {activeTab === "support" && <SupportScreen api={api} user={currentUser} />}
        {activeTab === "vehicles" && <VehiclesScreen api={api} user={currentUser} />}
        {activeTab === "account" && (
          <AccountScreen
            session={session}
            onLogout={() => {
              setCurrentUser(null);
              setSession(null);
            }}
          />
        )}
      </View>
      <View style={styles.tabs}>
        {availableTabs.map((tab) => {
          const active = activeTab === tab.key;
          return (
            <TouchableOpacity key={tab.key} style={[styles.tab, active && styles.activeTab]} onPress={() => setActiveTab(tab.key)}>
              <Text style={[styles.tabText, active && styles.activeTabText]}>{tab.label}</Text>
            </TouchableOpacity>
          );
        })}
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  root: {
    flex: 1,
    backgroundColor: colors.background
  },
  content: {
    flex: 1
  },
  loadingProfile: {
    flex: 1,
    justifyContent: "center",
    padding: 18
  },
  loadingTitle: {
    color: colors.text,
    fontSize: 18,
    fontWeight: "900",
    marginBottom: 12
  },
  logoutButton: {
    marginTop: 12,
    minHeight: 44,
    alignItems: "center",
    justifyContent: "center",
    borderRadius: 8,
    backgroundColor: colors.primary
  },
  logoutText: {
    color: colors.primaryText,
    fontWeight: "900"
  },
  tabs: {
    flexDirection: "row",
    gap: 8,
    padding: 12,
    borderTopWidth: 1,
    borderTopColor: colors.border,
    backgroundColor: colors.surface
  },
  tab: {
    flex: 1,
    minHeight: 44,
    alignItems: "center",
    justifyContent: "center",
    borderRadius: 8,
    backgroundColor: colors.muted
  },
  activeTab: {
    backgroundColor: colors.primary
  },
  tabText: {
    color: colors.textMuted,
    fontWeight: "700",
    fontSize: 13
  },
  activeTabText: {
    color: colors.primaryText
  }
});
