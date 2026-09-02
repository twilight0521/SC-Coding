import { useState } from 'react';
import { DiffEditor } from './DiffEditor';
import { Button } from './ui/Button';
import { GitCompare, CheckCircle, XCircle } from 'lucide-react';

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
