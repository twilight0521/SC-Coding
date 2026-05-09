import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore, TaskResult } from '../stores/appStore';
import { Button } from './ui/Button';
import { Card, CardHeader, CardTitle, CardContent } from './ui/Card';
import { Play, CheckCircle, XCircle, Loader2 } from 'lucide-react';

interface TaskRunnerPanelProps {
  projectId: string;
}

export function TaskRunnerPanel({ projectId }: TaskRunnerPanelProps) {
  const { tasks, executeTask, runTaskSequence } = useAppStore();
  const [selectedTaskIds, setSelectedTaskIds] = useState<string[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [results, setResults] = useState<TaskResult[]>([]);

  const toggleTask = (taskId: string) => {
    setSelectedTaskIds(prev =>
      prev.includes(taskId)
        ? prev.filter(id => id !== taskId)
        : [...prev, taskId]
    );
  };

  const runSelectedTasks = async () => {
    if (selectedTaskIds.length === 0) return;
    setIsRunning(true);
    try {
      const taskResults = await runTaskSequence(projectId, selectedTaskIds);
      setResults(taskResults);
    } catch (err) {
      console.error('Task execution failed:', err);
    } finally {
      setIsRunning(false);
    }
  };

  const runSingleTask = async (taskId: string) => {
    setIsRunning(true);
    try {
      const result = await executeTask(taskId);
      setResults(prev => [...prev, result]);
    } catch (err) {
      console.error('Task execution failed:', err);
    } finally {
      setIsRunning(false);
    }
  };

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Play size={16} />
            Task Runner
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex gap-2">
              <Button
                onClick={runSelectedTasks}
                disabled={selectedTaskIds.length === 0 || isRunning}
              >
                {isRunning ? (
                  <>
                    <Loader2 size={14} className="mr-1 animate-spin" />
                    Running...
                  </>
                ) : (
                  <>
                    <Play size={14} className="mr-1" />
                    Run Selected ({selectedTaskIds.length})
                  </>
                )}
              </Button>
              <Button
                variant="outline"
                onClick={() => setSelectedTaskIds([])}
              >
                Clear Selection
              </Button>
            </div>

            <div className="space-y-2">
              {tasks.map((task) => (
                <div
                  key={task.id}
                  className={`flex items-center gap-3 p-2 rounded border cursor-pointer ${
                    selectedTaskIds.includes(task.id)
                      ? 'border-primary bg-primary/5'
                      : 'border-muted hover:bg-muted'
                  }`}
                  onClick={() => toggleTask(task.id)}
                >
                  <input
                    type="checkbox"
                    checked={selectedTaskIds.includes(task.id)}
                    onChange={() => {}}
                    className="rounded"
                  />
                  <div className="flex-1">
                    <span className="text-sm font-medium">{task.title}</span>
                    <span className="text-xs text-muted-foreground ml-2">
                      {task.task_type}
                    </span>
                  </div>
                  <span
                    className={`text-xs px-2 py-1 rounded ${
                      task.status === 'completed'
                        ? 'bg-green-100 text-green-700'
                        : task.status === 'failed'
                        ? 'bg-red-100 text-red-700'
                        : 'bg-muted text-muted-foreground'
                    }`}
                  >
                    {task.status}
                  </span>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={(e) => {
                      e.stopPropagation();
                      runSingleTask(task.id);
                    }}
                  >
                    <Play size={12} />
                  </Button>
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {results.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Results</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {results.map((result, index) => (
                <div
                  key={index}
                  className={`p-3 rounded border ${
                    result.status === 'completed'
                      ? 'border-green-200 bg-green-50'
                      : 'border-red-200 bg-red-50'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      {result.status === 'completed' ? (
                        <CheckCircle size={14} className="text-green-600" />
                      ) : (
                        <XCircle size={14} className="text-red-600" />
                      )}
                      <span className="text-sm font-medium">
                        {result.task_id.slice(0, 8)}...
                      </span>
                    </div>
                    <div className="flex items-center gap-3 text-xs text-muted-foreground">
                      <span>Cost: ${result.cost.toFixed(4)}</span>
                      <span>{result.duration_ms}ms</span>
                    </div>
                  </div>
                  {result.error && (
                    <p className="text-xs text-red-600 mt-1">{result.error}</p>
                  )}
                  {result.output && (
                    <pre className="text-xs mt-2 p-2 bg-white rounded overflow-auto">
                      {result.output.slice(0, 200)}
                      {result.output.length > 200 && '...'}
                    </pre>
                  )}
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

// Decision log viewer
export function DecisionLogViewer({ projectRunId }: { projectRunId: string }) {
  const [logs, setLogs] = useState<Array<{
    id: string;
    decision_type: string;
    decision_summary: string;
    reason: string;
    decided_by: string;
    risk_level: string;
    created_at: string;
  }>>([]);
  const [loading, setLoading] = useState(false);

  const loadLogs = async () => {
    setLoading(true);
    try {
      const data = await invoke<Array<{
        id: string;
        decision_type: string;
        decision_summary: string;
        reason: string;
        decided_by: string;
        risk_level: string;
        created_at: string;
      }>>('get_decision_logs', { projectRunId });
      setLogs(data);
    } catch (err) {
      console.error('Failed to load decision logs:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Decision Log</CardTitle>
          <Button variant="outline" size="sm" onClick={loadLogs}>
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
        ) : logs.length === 0 ? (
          <p className="text-sm text-muted-foreground">No decisions logged</p>
        ) : (
          <div className="space-y-3">
            {logs.map((log) => (
              <div key={log.id} className="p-3 bg-muted rounded">
                <div className="flex items-center justify-between mb-1">
                  <span className="text-sm font-semibold">{log.decision_type}</span>
                  <span className={`text-xs px-2 py-1 rounded ${
                    log.risk_level === 'high'
                      ? 'bg-red-100 text-red-700'
                      : log.risk_level === 'medium'
                      ? 'bg-yellow-100 text-yellow-700'
                      : 'bg-green-100 text-green-700'
                  }`}>
                    {log.risk_level}
                  </span>
                </div>
                <p className="text-sm">{log.decision_summary}</p>
                <p className="text-xs text-muted-foreground mt-1">
                  Reason: {log.reason}
                </p>
                <div className="flex items-center justify-between mt-2 text-xs text-muted-foreground">
                  <span>By: {log.decided_by}</span>
                  <span>{new Date(log.created_at).toLocaleString()}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}