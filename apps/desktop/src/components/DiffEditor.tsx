import { useMemo } from 'react';
import { computeDiff, DiffSegment } from './DiffViewer';

interface DiffEditorProps {
  original: string;
  modified: string;
  view: 'split' | 'original' | 'modified';
}

function highlightLine(line: string): React.ReactNode {
  return <span>{line || ' '}</span>;
}

export function DiffEditor({ original, modified, view }: DiffEditorProps) {
  const diff = useMemo(() => computeDiff(original, modified), [original, modified]);

  if (view === 'original') {
    return (
      <div className="flex h-full">
        <div className="flex-1 overflow-auto font-mono text-sm">
          <table className="w-full border-collapse">
            <tbody>
              {original.split('\n').map((line, i) => (
                <tr key={i} className="border-b border-muted">
                  <td className="w-12 px-2 py-1 text-right text-muted-foreground select-none bg-muted">
                    {i + 1}
                  </td>
                  <td className="px-2 py-1 bg-white dark:bg-gray-900">
                    {highlightLine(line)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    );
  }

  if (view === 'modified') {
    return (
      <div className="flex h-full">
        <div className="flex-1 overflow-auto font-mono text-sm">
          <table className="w-full border-collapse">
            <tbody>
              {modified.split('\n').map((line, i) => (
                <tr key={i} className="border-b border-muted">
                  <td className="w-12 px-2 py-1 text-right text-muted-foreground select-none bg-muted">
                    {i + 1}
                  </td>
                  <td className="px-2 py-1 bg-blue-50 dark:bg-blue-900/20">
                    {highlightLine(line)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full">
      <div className="flex-1 overflow-auto font-mono text-sm border-r">
        <div className="sticky top-0 bg-muted px-2 py-1 text-xs font-semibold border-b">
          Original
        </div>
        <table className="w-full border-collapse">
          <tbody>
            {diff.map((segment: DiffSegment, si: number) => (
              segment.type === 'equal' ? (
                segment.lines.map((line: string, i: number) => (
                  <tr key={`${si}-${i}`} className="border-b border-muted">
                    <td className="w-12 px-2 py-1 text-right text-muted-foreground select-none">
                      {''}
                    </td>
                    <td className="px-2 py-1">
                      {highlightLine(line)}
                    </td>
                  </tr>
                ))
              ) : segment.type === 'delete' ? (
                segment.lines.map((line: string, i: number) => (
                  <tr key={`${si}-${i}`} className="border-b border-muted bg-red-50 dark:bg-red-900/20">
                    <td className="w-12 px-2 py-1 text-right text-muted-foreground select-none">
                      -
                    </td>
                    <td className="px-2 py-1 text-red-600 dark:text-red-400">
                      {highlightLine(line)}
                    </td>
                  </tr>
                ))
              ) : null
            ))}
          </tbody>
        </table>
      </div>

      <div className="flex-1 overflow-auto font-mono text-sm">
        <div className="sticky top-0 bg-muted px-2 py-1 text-xs font-semibold border-b">
          Modified
        </div>
        <table className="w-full border-collapse">
          <tbody>
            {diff.map((segment: DiffSegment, si: number) => (
              segment.type === 'equal' ? (
                segment.lines.map((line: string, i: number) => (
                  <tr key={`${si}-${i}`} className="border-b border-muted">
                    <td className="w-12 px-2 py-1 text-right text-muted-foreground select-none">
                      {''}
                    </td>
                    <td className="px-2 py-1">
                      {highlightLine(line)}
                    </td>
                  </tr>
                ))
              ) : segment.type === 'insert' ? (
                segment.lines.map((line: string, i: number) => (
                  <tr key={`${si}-${i}`} className="border-b border-muted bg-green-50 dark:bg-green-900/20">
                    <td className="w-12 px-2 py-1 text-right text-muted-foreground select-none">
                      +
                    </td>
                    <td className="px-2 py-1 text-green-600 dark:text-green-400">
                      {highlightLine(line)}
                    </td>
                  </tr>
                ))
              ) : null
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

export function DiffStats({ original, modified }: { original: string; modified: string }) {
  const diff = useMemo(() => computeDiff(original, modified), [original, modified]);

  const stats = useMemo(() => {
    let added = 0;
    let removed = 0;
    let unchanged = 0;

    diff.forEach((segment: DiffSegment) => {
      if (segment.type === 'equal') {
        unchanged += segment.lines.length;
      } else if (segment.type === 'insert') {
        added += segment.lines.length;
      } else if (segment.type === 'delete') {
        removed += segment.lines.length;
      }
    });

    return { added, removed, unchanged };
  }, [diff]);

  return (
    <div className="flex items-center gap-4 text-sm">
      <span className="text-green-600">+{stats.added}</span>
      <span className="text-red-600">-{stats.removed}</span>
      <span className="text-muted-foreground">{stats.unchanged} unchanged</span>
    </div>
  );
}
