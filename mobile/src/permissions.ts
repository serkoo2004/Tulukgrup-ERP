import type { CurrentUser, MobileTab } from "./types";

type PermissionAction = "view" | "create" | "update" | "approve";

const roleBypass = new Set(["admin", "manager", "operation"]);

export function canUse(user: CurrentUser | null, permissionKey: string, action: PermissionAction = "view") {
  if (!user) return false;
  if (roleBypass.has(user.role)) return true;
  return user.mobile_permissions.some((permission) => {
    if (permission.permission_key !== permissionKey) return false;
    if (action === "view") return permission.can_view;
    if (action === "create") return permission.can_create;
    if (action === "update") return permission.can_update;
    return permission.can_approve;
  });
}

export function tabPermission(tab: MobileTab) {
  if (tab === "dashboard") return "dashboard";
  if (tab === "inventory") return "inventory_view";
  if (tab === "support") return "support_ticket";
  if (tab === "vehicles") return "vehicle_view";
  return "account";
}

export function visibleTabs(user: CurrentUser | null, tabs: Array<{ key: MobileTab; label: string }>) {
  return tabs.filter((tab) => tab.key === "account" || canUse(user, tabPermission(tab.key), "view"));
}
