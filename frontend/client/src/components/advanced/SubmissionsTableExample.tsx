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
      accessorKey: 'filename',
      header: 'Filename',
      cell: ({ row }) => (
        <div className="font-medium">{row.getValue('filename')}</div>
      ),
    },
    {
      accessorKey: 'status',
      header: 'Status',
      cell: ({ row }) => {
        const status = row.getValue('status') as string;
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
      accessorKey: 'analysisType',
      header: 'Analysis',
    },
    {
      accessorKey: 'bountyAmount',
      header: 'Bounty',
      cell: ({ row }) => {
        const amount = parseFloat(row.getValue('bountyAmount'));
        return (
          <div className="font-medium">
            {amount.toFixed(2)} ETH
          </div>
        );
      },
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
    { key: 'filename', label: 'Filename' },
    { key: 'status', label: 'Status' },
    { key: 'submissionType', label: 'Type' },
    { key: 'analysisType', label: 'Analysis' },
    {
      key: 'bountyAmount',
      label: 'Bounty (ETH)',
      format: (value: string) => parseFloat(value).toFixed(2),
    },
    {
      key: 'createdAt',
      label: 'Created',
      format: (value: Date) => format(new Date(value), 'yyyy-MM-dd HH:mm:ss'),
    },
    { key: 'fileHash', label: 'File Hash' },
    { key: 'description', label: 'Description' },
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
        filename: row.filename,
        status: row.status,
        type: row.submissionType,
        analysis: row.analysisType,
        bounty: row.bountyAmount,
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