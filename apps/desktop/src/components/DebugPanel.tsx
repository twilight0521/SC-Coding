import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from './ui/Button';
import { Card, CardHeader, CardTitle, CardContent } from './ui/Card';
import { Bug, AlertTriangle, CheckCircle, XCircle, RotateCcw } from 'lucide-react';

interface DebugPanelProps {
  taskId: string;
}

interface DebugSession {
  session_id: string;
  task_id: string;
  status: string;
  current_round: number;
  max_rounds: number;
  fix_history: Array<{
    round: number;
    error_message: string;
    attempted_fix: string;
    files_modified: string[];
    success: boolean;
  }>;
  created_at: string;
}

export function DebugPanel({ taskId }: DebugPanelProps) {
  const [activeSession, setActiveSession] = useState<DebugSession | null>(null);
  const [loading, setLoading] = useState(false);

  const loadSessions = async () => {
    setLoading(true);
    try {
      const data = await invoke<DebugSession[]>('get_active_debug_sessions', { taskId });
      if (data.length > 0) {
        setActiveSession(data[0]);
      }
    } catch (err) {
      console.error('Failed to load debug sessions:', err);
    } finally {
      setLoading(false);
    }
  };

  const startSession = async () => {
    setLoading(true);
    try {
      const session = await invoke<DebugSession>('start_debug_session', {
        taskId,
        errorDescription: 'Test failed - needs debugging',
        maxRounds: 3,
      });
      setActiveSession(session);
    } catch (err) {
      console.error('Failed to start debug session:', err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Bug size={16} />
            Debug Session
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={loadSessions}
                disabled={loading}
              >
                <RotateCcw size={14} className="mr-1" />
                Load Sessions
              </Button>
              <Button onClick={startSession} disabled={loading}>
                <Bug size={14} className="mr-1" />
                Start New Session
              </Button>
            </div>

            {activeSession && (
              <div className="space-y-3 p-3 bg-muted rounded">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    {activeSession.status === 'success' ? (
                      <CheckCircle size={16} className="text-green-600" />
                    ) : activeSession.current_round >= activeSession.max_rounds ? (
                      <AlertTriangle size={16} className="text-yellow-600" />
                    ) : (
                      <Bug size={16} className="text-blue-600" />
                    )}
                    <span className="font-medium">
                      Round {activeSession.current_round}/{activeSession.max_rounds}
                    </span>
                  </div>
                  <span className="text-sm text-muted-foreground">
                    Status: {activeSession.status}
                  </span>
                </div>

                {activeSession.fix_history.length > 0 && (
                  <div className="space-y-2">
                    <h4 className="text-sm font-semibold">Fix History</h4>
                    {activeSession.fix_history.map((fix) => (
                      <div
                        key={fix.round}
                        className={`p-2 rounded border ${
                          fix.success
                            ? 'border-green-200 bg-green-50'
                            : 'border-red-200 bg-red-50'
                        }`}
                      >
                        <div className="flex items-center gap-2 mb-1">
                          <span className="text-xs font-semibold">
                            Round {fix.round}:
                          </span>
                          {fix.success ? (
                            <CheckCircle size={12} className="text-green-600" />
                          ) : (
                            <XCircle size={12} className="text-red-600" />
                          )}
                        </div>
                        <p className="text-xs text-muted-foreground">
                          Error: {fix.error_message}
                        </p>
                        <p className="text-xs mt-1">
                          Fix: {fix.attempted_fix}
                        </p>
                        {fix.files_modified.length > 0 && (
                          <p className="text-xs text-muted-foreground mt-1">
                            Modified: {fix.files_modified.join(', ')}
                          </p>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Error analyzer component
export function ErrorAnalyzer() {
  const [analysis, setAnalysis] = useState<{
    error_type: string;
    likely_cause: string;
    suggestion: string;
  } | null>(null);
  const [errorInput, setErrorInput] = useState('');

  const analyzeError = async () => {
    if (!errorInput.trim()) return;
    try {
      const result = await invoke<{
        error_type: string;
        likely_cause: string;
        suggestion: string;
      }>('analyze_error', {
        projectId: '',
        errorOutput: errorInput,
      });
      setAnalysis(result);
    } catch (err) {
      console.error('Analysis failed:', err);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <AlertTriangle size={16} />
          Error Analyzer
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          <textarea
            className="w-full p-2 border rounded text-sm font-mono h-24"
            placeholder="Paste error output here..."
            value={errorInput}
            onChange={(e) => setErrorInput(e.target.value)}
          />
          <Button onClick={analyzeError} disabled={!errorInput.trim()}>
            Analyze
          </Button>
          {analysis && (
            <div className="p-3 bg-muted rounded space-y-2">
              <div>
                <span className="text-xs font-semibold">Type:</span>
                <span className="ml-2 text-sm">{analysis.error_type}</span>
              </div>
              <div>
                <span className="text-xs font-semibold">Cause:</span>
                <p className="text-sm">{analysis.likely_cause}</p>
              </div>
              <div>
                <span className="text-xs font-semibold">Suggestion:</span>
                <p className="text-sm">{analysis.suggestion}</p>
              </div>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}