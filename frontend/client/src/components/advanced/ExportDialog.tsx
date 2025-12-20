/**
 * Export Dialog Component
 * User-friendly dialog for exporting data in multiple formats
 */

import React from 'react';
import { Download, FileText, FileJson, FileSpreadsheet, FileImage } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { exportData, type ExportColumn, type ExportOptions } from '@/lib/export';

interface ExportDialogProps<T> {
  data: T[];
  columns: ExportColumn[];
  defaultFilename?: string;
  title?: string;
  trigger?: React.ReactNode;
}

type ExportFormat = 'csv' | 'json' | 'pdf' | 'excel';

export function ExportDialog<T extends Record<string, any>>({
  data,
  columns,
  defaultFilename = 'export',
  title = 'Export Data',
  trigger,
}: ExportDialogProps<T>) {
  const [open, setOpen] = React.useState(false);
  const [format, setFormat] = React.useState<ExportFormat>('csv');
  const [filename, setFilename] = React.useState(defaultFilename);
  const [selectedColumns, setSelectedColumns] = React.useState<string[]>(
    columns.map((col) => col.key)
  );
  const [includeTimestamp, setIncludeTimestamp] = React.useState(true);

  const formatIcons: Record<ExportFormat, React.ReactNode> = {
    csv: <FileText className="h-5 w-5" />,
    json: <FileJson className="h-5 w-5" />,
    pdf: <FileImage className="h-5 w-5" />,
    excel: <FileSpreadsheet className="h-5 w-5" />,
  };

  const formatLabels: Record<ExportFormat, string> = {
    csv: 'CSV (.csv)',
    json: 'JSON (.json)',
    pdf: 'PDF (.pdf)',
    excel: 'Excel (.xlsx)',
  };

  const formatDescriptions: Record<ExportFormat, string> = {
    csv: 'Comma-separated values, compatible with Excel and spreadsheet applications',
    json: 'JavaScript Object Notation, ideal for developers and data processing',
    pdf: 'Portable Document Format, perfect for sharing and printing',
    excel: 'Microsoft Excel format with formatting and formulas',
  };

  const toggleColumn = (columnKey: string) => {
    setSelectedColumns((prev) =>
      prev.includes(columnKey)
        ? prev.filter((key) => key !== columnKey)
        : [...prev, columnKey]
    );
  };

  const handleExport = () => {
    const filteredColumns = columns.filter((col) =>
      selectedColumns.includes(col.key)
    );

    const options: ExportOptions = {
      filename,
      columns: filteredColumns,
      title,
      includeTimestamp,
    };

    exportData(data, format, options);
    setOpen(false);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        {trigger || (
          <Button variant="outline" size="sm">
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
        )}
      </DialogTrigger>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Export Data</DialogTitle>
          <DialogDescription>
            Choose format and customize your export
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* Format Selection */}
          <div className="space-y-3">
            <Label>Export Format</Label>
            <RadioGroup value={format} onValueChange={(value) => setFormat(value as ExportFormat)}>
              {(Object.keys(formatLabels) as ExportFormat[]).map((fmt) => (
                <div key={fmt} className="flex items-start space-x-3 space-y-0">
                  <RadioGroupItem value={fmt} id={fmt} />
                  <div className="flex-1 space-y-1">
                    <Label htmlFor={fmt} className="font-medium flex items-center gap-2">
                      {formatIcons[fmt]}
                      {formatLabels[fmt]}
                    </Label>
                    <p className="text-sm text-muted-foreground">
                      {formatDescriptions[fmt]}
                    </p>
                  </div>
                </div>
              ))}
            </RadioGroup>
          </div>

          {/* Filename */}
          <div className="space-y-2">
            <Label htmlFor="filename">Filename</Label>
            <Input
              id="filename"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              placeholder="Enter filename"
            />
            <p className="text-sm text-muted-foreground">
              File extension will be added automatically
            </p>
          </div>

          {/* Column Selection */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <Label>Columns to Export</Label>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  if (selectedColumns.length === columns.length) {
                    setSelectedColumns([]);
                  } else {
                    setSelectedColumns(columns.map((col) => col.key));
                  }
                }}
              >
                {selectedColumns.length === columns.length ? 'Deselect All' : 'Select All'}
              </Button>
            </div>
            <div className="grid grid-cols-2 gap-3 max-h-48 overflow-y-auto p-4 border rounded-md">
              {columns.map((column) => (
                <div key={column.key} className="flex items-center space-x-2">
                  <Checkbox
                    id={column.key}
                    checked={selectedColumns.includes(column.key)}
                    onCheckedChange={() => toggleColumn(column.key)}
                  />
                  <Label
                    htmlFor={column.key}
                    className="text-sm font-normal cursor-pointer"
                  >
                    {column.label}
                  </Label>
                </div>
              ))}
            </div>
          </div>

          {/* Options */}
          <div className="space-y-3">
            <Label>Options</Label>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="timestamp"
                checked={includeTimestamp}
                onCheckedChange={(checked) => setIncludeTimestamp(!!checked)}
              />
              <Label htmlFor="timestamp" className="text-sm font-normal cursor-pointer">
                Add timestamp to filename
              </Label>
            </div>
          </div>

          {/* Summary */}
          <div className="rounded-lg bg-muted p-4 space-y-2">
            <p className="text-sm font-medium">Export Summary</p>
            <div className="text-sm text-muted-foreground space-y-1">
              <div>Format: <span className="font-medium text-foreground">{formatLabels[format]}</span></div>
              <div>Rows: <span className="font-medium text-foreground">{data.length}</span></div>
              <div>Columns: <span className="font-medium text-foreground">{selectedColumns.length}</span></div>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button onClick={handleExport} disabled={selectedColumns.length === 0}>
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}