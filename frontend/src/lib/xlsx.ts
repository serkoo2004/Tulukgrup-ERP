import { strFromU8, unzipSync } from "fflate";

type ParsedCell = string | number | boolean | null;

export type ParsedXlsx = {
  sheetName: string;
  headers: string[];
  rows: Record<string, ParsedCell>[];
};

export async function parseFirstWorksheet(file: File): Promise<ParsedXlsx> {
  const bytes = new Uint8Array(await file.arrayBuffer());
  const zip = unzipSync(bytes);
  const sharedStrings = parseSharedStrings(readZipText(zip, "xl/sharedStrings.xml"));
  const workbook = parseWorkbook(readZipText(zip, "xl/workbook.xml"));
  const rels = parseWorkbookRels(readZipText(zip, "xl/_rels/workbook.xml.rels"));
  const firstSheet = workbook[0];
  if (!firstSheet) {
    throw new Error("Excel dosyasında sayfa bulunamadı");
  }

  const target = rels[firstSheet.relId];
  if (!target) {
    throw new Error("Excel sayfa referansı okunamadı");
  }

  const sheetPath = target.startsWith("xl/") ? target : `xl/${target.replace(/^\/?xl\//, "")}`;
  const sheetXml = readZipText(zip, sheetPath);
  const table = parseSheet(sheetXml, sharedStrings);
  const headers = (table[0] ?? []).map((cell, index) => normalizeHeader(cell, index));
  const rows = table.slice(1).map((row) => {
    const item: Record<string, ParsedCell> = {};
    headers.forEach((header, index) => {
      item[header] = row[index] ?? null;
    });
    return item;
  });

  return {
    sheetName: firstSheet.name,
    headers,
    rows
  };
}

function readZipText(zip: Record<string, Uint8Array>, path: string) {
  const entry = zip[path];
  if (!entry) return "";
  return strFromU8(entry);
}

function parseXml(xml: string) {
  if (!xml) return null;
  const parsed = new DOMParser().parseFromString(xml, "application/xml");
  const error = parsed.querySelector("parsererror");
  if (error) throw new Error("Excel XML okunamadı");
  return parsed;
}

function parseSharedStrings(xml: string) {
  const doc = parseXml(xml);
  if (!doc) return [];
  return Array.from(doc.getElementsByTagName("si")).map((node) =>
    Array.from(node.getElementsByTagName("t"))
      .map((textNode) => textNode.textContent ?? "")
      .join("")
  );
}

function parseWorkbook(xml: string) {
  const doc = parseXml(xml);
  if (!doc) return [];
  return Array.from(doc.getElementsByTagName("sheet")).map((node) => ({
    name: node.getAttribute("name") ?? "Sheet",
    relId: node.getAttribute("r:id") ?? ""
  }));
}

function parseWorkbookRels(xml: string) {
  const doc = parseXml(xml);
  if (!doc) return {} as Record<string, string>;
  const rels: Record<string, string> = {};
  Array.from(doc.getElementsByTagName("Relationship")).forEach((node) => {
    const id = node.getAttribute("Id");
    const target = node.getAttribute("Target");
    if (id && target) rels[id] = target;
  });
  return rels;
}

function parseSheet(xml: string, sharedStrings: string[]) {
  const doc = parseXml(xml);
  if (!doc) return [];
  const rows: ParsedCell[][] = [];

  Array.from(doc.getElementsByTagName("row")).forEach((rowNode) => {
    const row: ParsedCell[] = [];
    Array.from(rowNode.getElementsByTagName("c")).forEach((cellNode) => {
      const ref = cellNode.getAttribute("r") ?? "";
      const colIndex = columnRefToIndex(ref.replace(/\d+/g, ""));
      row[colIndex] = parseCellValue(cellNode, sharedStrings);
    });
    rows.push(row);
  });

  return rows;
}

function parseCellValue(cellNode: Element, sharedStrings: string[]): ParsedCell {
  const type = cellNode.getAttribute("t");
  if (type === "inlineStr") {
    return Array.from(cellNode.getElementsByTagName("t"))
      .map((node) => node.textContent ?? "")
      .join("");
  }

  const raw = cellNode.getElementsByTagName("v")[0]?.textContent ?? "";
  if (raw === "") return null;
  if (type === "s") return sharedStrings[Number(raw)] ?? "";
  if (type === "b") return raw === "1";
  if (type === "str") return raw;

  const numeric = Number(raw);
  return Number.isNaN(numeric) ? raw : numeric;
}

function columnRefToIndex(ref: string) {
  let index = 0;
  for (const char of ref.toUpperCase()) {
    index = index * 26 + (char.charCodeAt(0) - 64);
  }
  return Math.max(0, index - 1);
}

function normalizeHeader(value: ParsedCell, index: number) {
  const text = String(value ?? `column_${index + 1}`)
    .trim()
    .toLowerCase()
    .replaceAll("ı", "i")
    .replaceAll("ğ", "g")
    .replaceAll("ü", "u")
    .replaceAll("ş", "s")
    .replaceAll("ö", "o")
    .replaceAll("ç", "c")
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
  return text || `column_${index + 1}`;
}
