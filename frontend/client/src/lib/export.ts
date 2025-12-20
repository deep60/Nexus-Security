/**
 * Export Utilities
 * Support for CSV, JSON, and PDF exports
 */

import { parse, unparse } from 'papaparse';
import jsPDF from 'jspdf';
import autoTable from 'jspdf-autotable';
import * as XLSX from 'xlsx';

export interface ExportColumn {
  key: string;
  label: string;
  format?: (value: any) => string;
}

export interface ExportOptions {
  filename?: string;
  columns?: ExportColumn[];
  title?: string;
  includeTimestamp?: boolean;
}

/**
 * Export data to CSV format
 */
export function exportToCSV<T extends Record<string, any>>(
  data: T[],
  options: ExportOptions = {}
): void {
  const {
    filename = 'export.csv',
    columns,
    includeTimestamp = true,
  } = options;

  // Format data based on columns
  let formattedData = data;
  if (columns) {
    formattedData = data.map((row) => {
      const formatted: Record<string, any> = {};
      columns.forEach((col) => {
        const value = row[col.key];
        formatted[col.label] = col.format ? col.format(value) : value;
      });
      return formatted;
    });
  }

  // Convert to CSV
  const csv = unparse(formattedData);

  // Download
  const timestamp = includeTimestamp ? `_${Date.now()}` : '';
  const finalFilename = filename.replace(/\.csv$/, '') + timestamp + '.csv';
  downloadFile(csv, finalFilename, 'text/csv');
}

/**
 * Export data to JSON format
 */
export function exportToJSON<T extends Record<string, any>>(
  data: T[],
  options: ExportOptions = {}
): void {
  const {
    filename = 'export.json',
    columns,
    includeTimestamp = true,
  } = options;

  // Format data based on columns
  let formattedData = data;
  if (columns) {
    formattedData = data.map((row) => {
      const formatted: Record<string, any> = {};
      columns.forEach((col) => {
        const value = row[col.key];
        formatted[col.key] = col.format ? col.format(value) : value;
      });
      return formatted;
    });
  }

  // Convert to JSON (pretty print)
  const json = JSON.stringify(formattedData, null, 2);

  // Download
  const timestamp = includeTimestamp ? `_${Date.now()}` : '';
  const finalFilename = filename.replace(/\.json$/, '') + timestamp + '.json';
  downloadFile(json, finalFilename, 'application/json');
}

/**
 * Export data to PDF format
 */
export function exportToPDF<T extends Record<string, any>>(
  data: T[],
  options: ExportOptions = {}
): void {
  const {
    filename = 'export.pdf',
    columns,
    title = 'Export Report',
    includeTimestamp = true,
  } = options;

  const doc = new jsPDF();

  // Add title
  doc.setFontSize(18);
  doc.text(title, 14, 20);

  // Add timestamp
  doc.setFontSize(10);
  doc.text(`Generated: ${new Date().toLocaleString()}`, 14, 28);

  // Prepare table data
  const headers = columns
    ? columns.map((col) => col.label)
    : Object.keys(data[0] || {});

  const rows = data.map((row) => {
    if (columns) {
      return columns.map((col) => {
        const value = row[col.key];
        return col.format ? col.format(value) : String(value ?? '');
      });
    }
    return Object.values(row).map((val) => String(val ?? ''));
  });

  // Add table
  autoTable(doc, {
    head: [headers],
    body: rows,
    startY: 35,
    theme: 'grid',
    styles: {
      fontSize: 8,
      cellPadding: 2,
    },
    headStyles: {
      fillColor: [66, 66, 66],
      textColor: [255, 255, 255],
      fontStyle: 'bold',
    },
  });

  // Download
  const timestamp = includeTimestamp ? `_${Date.now()}` : '';
  const finalFilename = filename.replace(/\.pdf$/, '') + timestamp + '.pdf';
  doc.save(finalFilename);
}

/**
 * Export data to Excel format
 */
export function exportToExcel<T extends Record<string, any>>(
  data: T[],
  options: ExportOptions = {}
): void {
  const {
    filename = 'export.xlsx',
    columns,
    title,
    includeTimestamp = true,
  } = options;

  // Format data based on columns
  let formattedData = data;
  if (columns) {
    formattedData = data.map((row) => {
      const formatted: Record<string, any> = {};
      columns.forEach((col) => {
        const value = row[col.key];
        formatted[col.label] = col.format ? col.format(value) : value;
      });
      return formatted;
    });
  }

  // Create worksheet
  const worksheet = XLSX.utils.json_to_sheet(formattedData);

  // Create workbook
  const workbook = XLSX.utils.book_new();
  XLSX.utils.book_append_sheet(workbook, worksheet, title || 'Data');

  // Download
  const timestamp = includeTimestamp ? `_${Date.now()}` : '';
  const finalFilename = filename.replace(/\.xlsx$/, '') + timestamp + '.xlsx';
  XLSX.writeFile(workbook, finalFilename);
}

/**
 * Helper function to download a file
 */
function downloadFile(content: string, filename: string, type: string): void {
  const blob = new Blob([content], { type });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

/**
 * Export data in multiple formats
 */
export function exportData<T extends Record<string, any>>(
  data: T[],
  format: 'csv' | 'json' | 'pdf' | 'excel',
  options: ExportOptions = {}
): void {
  switch (format) {
    case 'csv':
      exportToCSV(data, options);
      break;
    case 'json':
      exportToJSON(data, options);
      break;
    case 'pdf':
      exportToPDF(data, options);
      break;
    case 'excel':
      exportToExcel(data, options);
      break;
    default:
      throw new Error(`Unsupported export format: ${format}`);
  }
}