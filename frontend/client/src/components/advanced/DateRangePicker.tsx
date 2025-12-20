/**
 * Date Range Picker Component
 * Features: Date range selection, presets, calendar widget
 */

import React from 'react';
import { format, subDays, startOfDay, endOfDay } from 'date-fns';
import { Calendar as CalendarIcon } from 'lucide-react';
import { DateRange } from 'react-day-picker';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Calendar } from '@/components/ui/calendar';
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

interface DateRangePickerProps {
  value?: DateRange;
  onChange: (range: DateRange | undefined) => void;
  className?: string;
}

const DATE_PRESETS = [
  { label: 'Today', value: 'today' },
  { label: 'Yesterday', value: 'yesterday' },
  { label: 'Last 7 days', value: 'last7days' },
  { label: 'Last 14 days', value: 'last14days' },
  { label: 'Last 30 days', value: 'last30days' },
  { label: 'Last 90 days', value: 'last90days' },
  { label: 'This month', value: 'thisMonth' },
  { label: 'Last month', value: 'lastMonth' },
  { label: 'Custom', value: 'custom' },
];

export function DateRangePicker({ value, onChange, className }: DateRangePickerProps) {
  const [selectedPreset, setSelectedPreset] = React.useState<string>('');

  const applyPreset = (preset: string) => {
    const today = new Date();
    let range: DateRange | undefined;

    switch (preset) {
      case 'today':
        range = {
          from: startOfDay(today),
          to: endOfDay(today),
        };
        break;
      case 'yesterday':
        range = {
          from: startOfDay(subDays(today, 1)),
          to: endOfDay(subDays(today, 1)),
        };
        break;
      case 'last7days':
        range = {
          from: startOfDay(subDays(today, 6)),
          to: endOfDay(today),
        };
        break;
      case 'last14days':
        range = {
          from: startOfDay(subDays(today, 13)),
          to: endOfDay(today),
        };
        break;
      case 'last30days':
        range = {
          from: startOfDay(subDays(today, 29)),
          to: endOfDay(today),
        };
        break;
      case 'last90days':
        range = {
          from: startOfDay(subDays(today, 89)),
          to: endOfDay(today),
        };
        break;
      case 'thisMonth':
        range = {
          from: startOfDay(new Date(today.getFullYear(), today.getMonth(), 1)),
          to: endOfDay(today),
        };
        break;
      case 'lastMonth':
        const lastMonth = new Date(today.getFullYear(), today.getMonth() - 1, 1);
        const lastMonthEnd = new Date(today.getFullYear(), today.getMonth(), 0);
        range = {
          from: startOfDay(lastMonth),
          to: endOfDay(lastMonthEnd),
        };
        break;
      case 'custom':
        range = undefined;
        break;
      default:
        range = undefined;
    }

    setSelectedPreset(preset);
    onChange(range);
  };

  const formatDateRange = (range: DateRange | undefined) => {
    if (!range?.from) return 'Pick a date range';
    if (!range.to) return format(range.from, 'LLL dd, y');
    return `${format(range.from, 'LLL dd, y')} - ${format(range.to, 'LLL dd, y')}`;
  };

  return (
    <div className={cn('flex gap-2', className)}>
      {/* Preset Select */}
      <Select value={selectedPreset} onValueChange={applyPreset}>
        <SelectTrigger className="w-[180px]">
          <SelectValue placeholder="Select preset" />
        </SelectTrigger>
        <SelectContent>
          {DATE_PRESETS.map((preset) => (
            <SelectItem key={preset.value} value={preset.value}>
              {preset.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>

      {/* Calendar Popover */}
      <Popover>
        <PopoverTrigger asChild>
          <Button
            id="date"
            variant="outline"
            className={cn(
              'w-[300px] justify-start text-left font-normal',
              !value && 'text-muted-foreground'
            )}
          >
            <CalendarIcon className="mr-2 h-4 w-4" />
            {formatDateRange(value)}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0" align="start">
          <Calendar
            initialFocus
            mode="range"
            defaultMonth={value?.from}
            selected={value}
            onSelect={(range) => {
              onChange(range);
              setSelectedPreset('custom');
            }}
            numberOfMonths={2}
          />
        </PopoverContent>
      </Popover>

      {/* Clear Button */}
      {value?.from && (
        <Button
          variant="ghost"
          size="sm"
          onClick={() => {
            onChange(undefined);
            setSelectedPreset('');
          }}
        >
          Clear
        </Button>
      )}
    </div>
  );
}