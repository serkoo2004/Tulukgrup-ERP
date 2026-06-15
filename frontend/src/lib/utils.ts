import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatDate(value?: string | null) {
  if (!value) return "-";
  return new Intl.DateTimeFormat("tr-TR").format(new Date(value));
}

export function formatDateTime(value?: string | null) {
  if (!value) return "-";
  return new Intl.DateTimeFormat("tr-TR", {
    dateStyle: "short",
    timeStyle: "short"
  }).format(new Date(value));
}

export function formatMoney(value?: string | number | null) {
  if (value === null || value === undefined || value === "") return "0 TL";
  const numberValue = typeof value === "number" ? value : Number(value);
  if (Number.isNaN(numberValue)) return String(value);
  return new Intl.NumberFormat("tr-TR", {
    style: "currency",
    currency: "TRY",
    maximumFractionDigits: 0
  }).format(numberValue);
}

export function formatQuantity(value?: string | number | null, unit?: string | null) {
  if (value === null || value === undefined || value === "") return unit ? `0 ${unit}` : "0";
  const numberValue = typeof value === "number" ? value : Number(value);
  if (Number.isNaN(numberValue)) return String(value);
  const formatted = new Intl.NumberFormat("tr-TR", {
    maximumFractionDigits: 4
  }).format(numberValue);
  return unit ? `${formatted} ${unit}` : formatted;
}

export function formatQuantityInput(value?: string | number | null) {
  if (value === null || value === undefined || value === "") return "";
  const numberValue = typeof value === "number" ? value : Number(value);
  if (Number.isNaN(numberValue)) return String(value);
  return Number.isInteger(numberValue) ? String(numberValue) : String(numberValue).replace(/(\.\d*?)0+$/, "$1").replace(/\.$/, "");
}

export function humanize(value?: string | null) {
  if (!value) return "-";
  return value
    .replaceAll("_", " ")
    .replace(/\b\w/g, (letter: string) => letter.toLocaleUpperCase("tr-TR"));
}
