import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './ui/Button';
import { Card, CardHeader, CardTitle, CardContent } from './ui/Card';
import { DiffViewer } from './DiffViewer';
import { FileDiff, CheckCircle, XCircle, Loader2 } from 'lucide-react';

interface FileChange {
  id: string;
  task_id: string;
  file_path: string;
  change_type: string;
  status: string;
  created_at: string;
}

interface FileChangePreview {
  file_path: string;
  original: string;
  modified: string;
}

export function FileChangesPanel({ projectId }: { projectId: string }) {
  const [changes, setChanges] = useState<FileChange[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [preview, setPreview] = useState<FileChangePreview | null>(null);
  const [loading, setLoading] = useState(false);
  const [loadingPreview, setLoadingPreview] = useState(false);

  const loadChanges = async () => {
    setLoading(true);
    try {
      const data = await invoke<FileChange[]>('list_pending_file_changes', { projectId });
      setChanges(data);
      if (selectedId && !data.some((c) => c.id === selectedId)) {
        setSelectedId(null);
        setPreview(null);
      }
    } catch (err) {
      console.error('Failed to load file changes:', err);
    } finally {
      setLoading(false);
    }
  };

  const selectChange = async (changeId: string) => {
    setSelectedId(changeId);
    setLoadingPreview(true);
    try {
      const data = await invoke<FileChangePreview>('preview_file_change', { changeId });
      setPreview(data);
    } catch (err) {
      console.error('Failed to preview change:', err);
      setPreview(null);
    } finally {
      setLoadingPreview(false);
    }
  };

  const applyChange = async () => {
    if (!selectedId) return;
    try {
      await invoke('apply_file_change', { changeId: selectedId });
      setSelectedId(null);
      setPreview(null);
      await loadChanges();
    } catch (err) {
      console.error('Failed to apply change:', err);
    }
  };

  const rejectChange = async () => {
    if (!selectedId) return;
    try {
      await invoke('reject_file_change', { changeId: selectedId });
      setSelectedId(null);
      setPreview(null);
      await loadChanges();
    } catch (err) {
      console.error('Failed to reject change:', err);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <FileDiff size={16} />
              Pending Changes
            </CardTitle>
            <Button variant="outline" size="sm" onClick={loadChanges}>
              <Loader2 size={14} className={`mr-1 ${loading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="flex items-center justify-center py-4">
              <Loader2 size={16} className="animate-spin" />
            </div>
          ) : changes.length === 0 ? (
            <p className="text-sm text-muted-foreground py-4 text-center">
              No pending changes. Run a task to generate patches.
            </p>
          ) : (
            <div className="space-y-2">
              {changes.map((change) => (
                <div
                  key={change.id}
                  className={`flex items-center gap-2 p-2 rounded border cursor-pointer ${
                    selectedId === change.id
                      ? 'border-primary bg-primary/5'
                      : 'border-muted hover:bg-muted'
                  }`}
                  onClick={() => selectChange(change.id)}
                >
                  <span className={`text-xs px-2 py-0.5 rounded ${
                    change.change_type === 'create'
                      ? 'bg-green-100 text-green-700'
                      : change.change_type === 'delete'
                      ? 'bg-red-100 text-red-700'
                      : 'bg-blue-100 text-blue-700'
                  }`}>
                    {change.change_type}
                  </span>
                  <span className="text-sm truncate flex-1">{change.file_path}</span>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {selectedId && (
        <div className="flex-1 flex flex-col overflow-hidden border rounded mt-3">
          <div className="flex items-center justify-between p-2 border-b bg-muted">
            <span className="text-sm font-mono truncate">{preview?.file_path || ''}</span>
            <div className="flex items-center gap-2">
              <Button variant="outline" size="sm" onClick={rejectChange}>
                <XCircle size={14} className="mr-1" />
                Reject
              </Button>
              <Button size="sm" onClick={applyChange}>
                <CheckCircle size={14} className="mr-1" />
                Apply
              </Button>
            </div>
          </div>
          {loadingPreview ? (
            <div className="flex-1 flex items-center justify-center text-muted-foreground">
              <Loader2 size={20} className="animate-spin" />
            </div>
          ) : preview ? (
            <div className="flex-1 overflow-hidden">
              <DiffViewer
                originalContent={preview.original}
                modifiedContent={preview.modified}
                filePath={preview.file_path}
              />
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center text-muted-foreground">
              <p className="text-sm">Unable to preview this change.</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
