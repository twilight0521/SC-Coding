import { useState } from 'react';
import { DiffEditor } from './DiffEditor';
import { Button } from './ui/Button';
import { GitCompare, CheckCircle, XCircle } from 'lucide-react';

export interface DiffSegment {
  type: 'equal' | 'insert' | 'delete';
  lines: string[];
}

interface DiffViewerProps {
  originalContent?: string | null;
  modifiedContent?: string | null;
  filePath?: string | null;
  onApply?: (content: string) => void;
  onReject?: () => void;
}

export function DiffViewer({ originalContent, modifiedContent, filePath, onApply, onReject }: DiffViewerProps) {
  const [view, setView] = useState<'split' | 'original' | 'modified'>('split');
  const [showDiff, setShowDiff] = useState(false);

  if (!filePath && !originalContent && !modifiedContent) {
    return (
      <div className="flex-1 flex items-center justify-center text-muted-foreground">
        <div className="text-center">
          <GitCompare size={48} className="mx-auto mb-4 opacity-50" />
          <p>No changes to display</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Toolbar */}
      <div className="flex items-center justify-between p-2 border-b bg-muted">
        <div className="flex items-center gap-2">
          <span className="text-sm font-mono">{filePath || 'Untitled'}</span>
        </div>
        <div className="flex items-center gap-2">
          <div className="flex rounded-md overflow-hidden border">
            <button
              className={`px-3 py-1 text-sm ${view === 'original' ? 'bg-primary text-primary-foreground' : 'bg-background'}`}
              onClick={() => setView('original')}
            >
              Original
            </button>
            <button
              className={`px-3 py-1 text-sm ${view === 'split' ? 'bg-primary text-primary-foreground' : 'bg-background'}`}
              onClick={() => setView('split')}
            >
              Split
            </button>
            <button
              className={`px-3 py-1 text-sm ${view === 'modified' ? 'bg-primary text-primary-foreground' : 'bg-background'}`}
              onClick={() => setView('modified')}
            >
              Modified
            </button>
          </div>
          {showDiff && (
            <button
              className="p-2 rounded hover:bg-accent"
              onClick={() => setShowDiff(false)}
              title="Show unified diff"
            >
              <GitCompare size={16} />
            </button>
          )}
        </div>
      </div>

      {/* Diff Content */}
      <div className="flex-1 overflow-auto">
        {originalContent && modifiedContent ? (
          <DiffEditor
            original={originalContent}
            modified={modifiedContent}
            view={view}
          />
        ) : (
          <pre className="p-4 text-sm font-mono whitespace-pre-wrap">
            {modifiedContent || originalContent || ''}
          </pre>
        )}
      </div>

      {/* Actions */}
      {(onApply || onReject) && (
        <div className="flex items-center justify-end gap-2 p-2 border-t">
          {onReject && (
            <Button variant="outline" size="sm" onClick={onReject}>
              <XCircle size={14} className="mr-1" />
              Reject
            </Button>
          )}
          {onApply && (
            <Button size="sm" onClick={() => onApply(modifiedContent || '')}>
              <CheckCircle size={14} className="mr-1" />
              Apply Changes
            </Button>
          )}
        </div>
      )}
    </div>
  );
}

// Simple diff algorithm for text comparison
export function computeDiff(original: string, modified: string): DiffSegment[] {
  const originalLines = original.split('\n');
  const modifiedLines = modified.split('\n');

  const result: DiffSegment[] = [];

  let i = 0, j = 0;

  while (i < originalLines.length || j < modifiedLines.length) {
    if (i < originalLines.length && j < modifiedLines.length) {
      if (originalLines[i] === modifiedLines[j]) {
        result.push({ type: 'equal', lines: [originalLines[i]] });
        i++;
        j++;
      } else {
        let nextMatchI = -1;
        let nextMatchJ = -1;

        for (let k = i; k < originalLines.length; k++) {
          for (let l = j; l < modifiedLines.length; l++) {
            if (originalLines[k] === modifiedLines[l]) {
              nextMatchI = k;
              nextMatchJ = l;
              break;
            }
          }
          if (nextMatchI !== -1) break;
        }

        if (nextMatchI === -1) {
          const deleted: string[] = [];
          while (i < originalLines.length) {
            deleted.push(originalLines[i]);
            i++;
          }
          const inserted: string[] = [];
          while (j < modifiedLines.length) {
            inserted.push(modifiedLines[j]);
            j++;
          }
          if (deleted.length > 0) result.push({ type: 'delete', lines: deleted });
          if (inserted.length > 0) result.push({ type: 'insert', lines: inserted });
        } else {
          if (nextMatchI > i) {
            result.push({ type: 'delete', lines: originalLines.slice(i, nextMatchI) });
            i = nextMatchI;
          }
          if (nextMatchJ > j) {
            result.push({ type: 'insert', lines: modifiedLines.slice(j, nextMatchJ) });
            j = nextMatchJ;
          }
        }
      }
    } else if (i < originalLines.length) {
      result.push({ type: 'delete', lines: originalLines.slice(i) });
      i = originalLines.length;
    } else if (j < modifiedLines.length) {
      result.push({ type: 'insert', lines: modifiedLines.slice(j) });
      j = modifiedLines.length;
    }
  }

  return result;
}
