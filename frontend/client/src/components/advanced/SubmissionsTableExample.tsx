/**
 * Example: Advanced Submissions Table
 * Demonstrates usage of DataTable, DateRangePicker, and ExportDialog
 */

import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { type ColumnDef } from '@tanstack/react-table';
import { format } from 'date-fns';
import { DateRange } from 'react-day-picker';
import { Badge } from '@/components/ui/badge';
import { DataTable } from './DataTable';
import { DateRangePicker } from './DateRangePicker';
import { ExportDialog } from './ExportDialog';
import { type Submission } from '@shared/schema';

export function SubmissionsTableExample() {
  const [dateRange, setDateRange] = React.useState<DateRange | undefined>();

  // Fetch submissions
  const { data: submissions = [], isLoading } = useQuery<Submission[]>({
    queryKey: ['/api/submissions'],
  });

  // Filter by date range
  const filteredSubmissions = React.useMemo(() => {
    if (!dateRange?.from) return submissions;

    return submissions.filter((sub) => {
      const subDate = new Date(sub.createdAt!);
      const from = dateRange.from!;
      const to = dateRange.to || dateRange.from!;

      return subDate >= from && subDate <= to;
    });
  }, [submissions, dateRange]);

  // Define table columns
  const columns: ColumnDef<Submission>[] = [
    {
      accessorKey: 'originalFilename',
      header: 'Filename',
      cell: ({ row }) => (
        <div className="font-medium">{row.getValue('originalFilename')}</div>
      ),
    },
    {
      accessorKey: 'analysisStatus',
      header: 'Status',
      cell: ({ row }) => {
        const status = row.getValue('analysisStatus') as string;
        return (
          <Badge
            variant={
              status === 'completed'
                ? 'default'
                : status === 'analyzing'
                ? 'secondary'
                : 'outline'
            }
          >
            {status}
          </Badge>
        );
      },
    },
    {
      accessorKey: 'submissionType',
      header: 'Type',
    },
    {
      accessorKey: 'mimeType',
      header: 'File Type',
    },
    {
      accessorKey: 'createdAt',
      header: 'Created',
      cell: ({ row }) => {
        const date = row.getValue('createdAt') as Date;
        return (
          <div className="text-sm">
            {format(new Date(date), 'MMM dd, yyyy HH:mm')}
          </div>
        );
      },
    },
  ];

  // Export columns configuration
  const exportColumns = [
    { key: 'originalFilename', label: 'Filename' },
    { key: 'analysisStatus', label: 'Status' },
    { key: 'submissionType', label: 'Type' },
    { key: 'mimeType', label: 'File Type' },
    {
      key: 'createdAt',
      label: 'Created',
      format: (value: unknown) => format(new Date(value as Date), 'yyyy-MM-dd HH:mm:ss'),
    },
    { key: 'fileHash', label: 'File Hash' },
  ];

  // Handle bulk actions
  const handleBulkAction = (selectedRows: Submission[], action: string) => {
    console.log('Bulk action:', action, 'on', selectedRows.length, 'rows');

    if (action === 'delete') {
      // Implement bulk delete
      alert(`Delete ${selectedRows.length} submissions?`);
    } else if (action === 'export') {
      // Export selected rows
      const data = selectedRows.map((row) => ({
        filename: row.originalFilename,
        status: row.analysisStatus,
        type: row.submissionType,
        fileType: row.mimeType,
        created: format(new Date(row.createdAt!), 'yyyy-MM-dd HH:mm:ss'),
      }));

      // Could use ExportDialog or direct export here
      console.log('Exporting:', data);
    }
  };

  if (isLoading) {
    return <div>Loading...</div>;
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Submissions</h2>
          <p className="text-muted-foreground">
            Manage and analyze file submissions
          </p>
        </div>

        {/* Export Dialog */}
        <ExportDialog
          data={filteredSubmissions}
          columns={exportColumns}
          defaultFilename="submissions"
          title="Nexus Security - Submissions Report"
        />
      </div>

      {/* Date Range Filter */}
      <div className="flex items-center gap-4">
        <DateRangePicker
          value={dateRange}
          onChange={setDateRange}
        />
        {dateRange?.from && (
          <p className="text-sm text-muted-foreground">
            Showing {filteredSubmissions.length} of {submissions.length} submissions
          </p>
        )}
      </div>

      {/* Data Table */}
      <DataTable
        columns={columns}
        data={filteredSubmissions}
        onBulkAction={handleBulkAction}
        enableRowSelection
        enableColumnFilters
        enableSorting
        enablePagination
        pageSize={20}
      />
    </div>
  );
}