import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './ui/Button';
import { Card, CardHeader, CardTitle, CardContent } from './ui/Card';
import { Play, RotateCcw, CheckCircle, XCircle, Loader2 } from 'lucide-react';

interface TestRunnerProps {
  projectId: string;
}

export function TestRunner({ projectId }: TestRunnerProps) {
  const [isRunning, setIsRunning] = useState(false);
  const [testFilter, setTestFilter] = useState('');
  const [lastResult, setLastResult] = useState<{
    status: string;
    output: string;
    duration_ms: number;
    passed: boolean;
  } | null>(null);

  const runTests = async () => {
    setIsRunning(true);
    try {
      const result = await invoke<{
        status: string;
        output: string;
        duration_ms: number;
        passed: boolean;
      }>('run_tests', {
        projectId,
        testFilter: testFilter || null,
      });
      setLastResult(result);
    } catch (err) {
      console.error('Test failed:', err);
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
            Test Runner
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex gap-2">
              <input
                type="text"
                placeholder="Test filter (optional)"
                value={testFilter}
                onChange={(e) => setTestFilter(e.target.value)}
                className="flex-1 px-3 py-2 border rounded text-sm"
              />
              <Button onClick={runTests} disabled={isRunning}>
                {isRunning ? (
                  <>
                    <Loader2 size={14} className="mr-1 animate-spin" />
                    Running...
                  </>
                ) : (
                  <>
                    <Play size={14} className="mr-1" />
                    Run Tests
                  </>
                )}
              </Button>
            </div>

            {lastResult && (
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  {lastResult.passed ? (
                    <span className="flex items-center gap-1 text-green-600">
                      <CheckCircle size={14} />
                      Passed
                    </span>
                  ) : (
                    <span className="flex items-center gap-1 text-red-600">
                      <XCircle size={14} />
                      Failed
                    </span>
                  )}
                  <span className="text-sm text-muted-foreground">
                    {lastResult.duration_ms}ms
                  </span>
                </div>
                <pre className="p-3 bg-muted rounded text-xs font-mono overflow-auto max-h-48">
                  {lastResult.output}
                </pre>
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Test results panel showing history
export function TestHistory({ projectId }: { projectId: string }) {
  const [history, setHistory] = useState<Array<{
    id: string;
    command: string;
    start_time: string;
    status: string;
  }>>([]);
  const [loading, setLoading] = useState(false);

  const loadHistory = async () => {
    setLoading(true);
    try {
      const data = await invoke<Array<{
        id: string;
        command: string;
        start_time: string;
        status: string;
      }>>('get_test_history', { projectId, limit: 10 });
      setHistory(data);
    } catch (err) {
      console.error('Failed to load test history:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle>Test History</CardTitle>
          <Button variant="outline" size="sm" onClick={loadHistory}>
            <RotateCcw size={14} className="mr-1" />
            Refresh
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {loading ? (
          <div className="flex items-center justify-center py-4">
            <Loader2 size={16} className="animate-spin" />
          </div>
        ) : history.length === 0 ? (
          <p className="text-sm text-muted-foreground">No test history</p>
        ) : (
          <div className="space-y-2">
            {history.map((test) => (
              <div
                key={test.id}
                className="flex items-center justify-between p-2 bg-muted rounded"
              >
                <div>
                  <span className="text-sm font-mono">{test.command}</span>
                  <span className="text-xs text-muted-foreground ml-2">
                    {new Date(test.start_time).toLocaleString()}
                  </span>
                </div>
                <span
                  className={`text-xs ${
                    test.status === 'passed' ? 'text-green-600' : 'text-red-600'
                  }`}
                >
                  {test.status}
                </span>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}